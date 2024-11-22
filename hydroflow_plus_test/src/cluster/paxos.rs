use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

use hydroflow_plus::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use stream::NoOrder;
use tokio::time::Instant;

use super::quorum::collect_quorum;
use super::request_response::join_responses;

pub struct Proposer {}
pub struct Acceptor {}

pub trait PaxosPayload: Serialize + DeserializeOwned + PartialEq + Eq + Clone + Debug {}
impl<T: Serialize + DeserializeOwned + PartialEq + Eq + Clone + Debug> PaxosPayload for T {}

#[derive(Serialize, Deserialize, PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct Ballot {
    pub num: u32,
    pub proposer_id: ClusterId<Proposer>,
}

impl Ord for Ballot {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.num
            .cmp(&other.num)
            .then_with(|| self.proposer_id.raw_id.cmp(&other.proposer_id.raw_id))
    }
}

impl PartialOrd for Ballot {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LogValue<P> {
    ballot: Ballot,
    value: Option<P>, // might be a hole
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
struct P2a<P> {
    ballot: Ballot,
    slot: usize,
    value: Option<P>, // might be a re-committed hole
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "internal paxos code // TODO"
)]
pub fn paxos_core<'a, P: PaxosPayload, R>(
    proposers: &Cluster<'a, Proposer>,
    acceptors: &Cluster<'a, Acceptor>,
    r_to_acceptors_checkpoint: Stream<
        (ClusterId<R>, usize),
        Cluster<'a, Acceptor>,
        Unbounded,
        NoOrder,
    >,
    c_to_proposers: Stream<P, Cluster<'a, Proposer>, Unbounded>,
    f: usize,
    i_am_leader_send_timeout: u64,
    i_am_leader_check_timeout: u64,
    i_am_leader_check_timeout_delay_multiplier: usize,
) -> (
    Stream<Ballot, Cluster<'a, Proposer>, Unbounded>,
    Stream<(usize, Option<P>), Cluster<'a, Proposer>, Unbounded, NoOrder>,
) {
    proposers
        .source_iter(q!(["Proposers say hello"]))
        .for_each(q!(|s| println!("{}", s)));

    acceptors
        .source_iter(q!(["Acceptors say hello"]))
        .for_each(q!(|s| println!("{}", s)));

    let proposer_tick = proposers.tick();
    let acceptor_tick = acceptors.tick();

    let (sequencing_max_ballot_complete_cycle, sequencing_max_ballot_forward_reference) =
        proposers.forward_ref::<Stream<Ballot, _, _, NoOrder>>();
    let (a_log_complete_cycle, a_log_forward_reference) =
        acceptor_tick.forward_ref::<Singleton<_, _, _>>();

    let (p_ballot, p_is_leader, p_relevant_p1bs, a_max_ballot) = leader_election(
        proposers,
        acceptors,
        &proposer_tick,
        &acceptor_tick,
        f,
        i_am_leader_send_timeout,
        i_am_leader_check_timeout,
        i_am_leader_check_timeout_delay_multiplier,
        sequencing_max_ballot_forward_reference,
        a_log_forward_reference,
    );

    let just_became_leader = p_is_leader
        .clone()
        .continue_unless(p_is_leader.clone().defer_tick());

    let (p_to_replicas, a_log, sequencing_max_ballots) = sequence_payload(
        proposers,
        acceptors,
        &proposer_tick,
        &acceptor_tick,
        c_to_proposers,
        r_to_acceptors_checkpoint,
        p_ballot.clone(),
        p_is_leader,
        p_relevant_p1bs,
        f,
        a_max_ballot,
    );

    a_log_complete_cycle.complete(a_log);
    sequencing_max_ballot_complete_cycle.complete(sequencing_max_ballots);

    (
        // Only tell the clients once when leader election concludes
        just_became_leader.then(p_ballot).all_ticks(),
        p_to_replicas,
    )
}

#[expect(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "internal paxos code // TODO"
)]
fn leader_election<'a, L: Clone + Debug + Serialize + DeserializeOwned>(
    proposers: &Cluster<'a, Proposer>,
    acceptors: &Cluster<'a, Acceptor>,
    proposer_tick: &Tick<Cluster<'a, Proposer>>,
    acceptor_tick: &Tick<Cluster<'a, Acceptor>>,
    f: usize,
    i_am_leader_send_timeout: u64,
    i_am_leader_check_timeout: u64,
    i_am_leader_check_timeout_delay_multiplier: usize,
    p_received_p2b_ballots: Stream<Ballot, Cluster<'a, Proposer>, Unbounded, NoOrder>,
    a_log: Singleton<L, Tick<Cluster<'a, Acceptor>>, Bounded>,
) -> (
    Singleton<Ballot, Tick<Cluster<'a, Proposer>>, Bounded>,
    Optional<(), Tick<Cluster<'a, Proposer>>, Bounded>,
    Stream<L, Tick<Cluster<'a, Proposer>>, Bounded, NoOrder>,
    Singleton<Ballot, Tick<Cluster<'a, Acceptor>>, Bounded>,
) {
    let (p1b_fail_complete, p1b_fail) =
        proposers.forward_ref::<Stream<Ballot, _, Unbounded, NoOrder>>();
    let (p_to_proposers_i_am_leader_complete_cycle, p_to_proposers_i_am_leader_forward_ref) =
        proposers.forward_ref::<Stream<_, _, _, NoOrder>>();
    let (p_is_leader_complete_cycle, p_is_leader_forward_ref) =
        proposer_tick.forward_ref::<Optional<(), _, _>>();
    // a_to_proposers_p2b.clone().for_each(q!(|(_, p2b): (u32, P2b)| println!("Proposer received P2b: {:?}", p2b)));
    // p_to_proposers_i_am_leader.clone().for_each(q!(|ballot: Ballot| println!("Proposer received I am leader: {:?}", ballot)));
    // c_to_proposers.clone().for_each(q!(|payload: ClientPayload| println!("Client sent proposer payload: {:?}", payload)));

    let p_received_max_ballot = p1b_fail
        .union(p_received_p2b_ballots)
        .union(p_to_proposers_i_am_leader_forward_ref)
        .max()
        .unwrap_or(proposers.singleton(q!(Ballot {
            num: 0,
            proposer_id: ClusterId::from_raw(0)
        })));

    let (p_ballot, p_has_largest_ballot) = p_ballot_calc(
        proposer_tick,
        p_received_max_ballot.latest_tick(proposer_tick),
    );

    let (p_to_proposers_i_am_leader, p_trigger_election) = p_leader_heartbeat(
        proposers,
        proposer_tick,
        p_is_leader_forward_ref,
        p_ballot.clone(),
        i_am_leader_send_timeout,
        i_am_leader_check_timeout,
        i_am_leader_check_timeout_delay_multiplier,
    );

    p_to_proposers_i_am_leader_complete_cycle.complete(p_to_proposers_i_am_leader);

    let p_to_acceptors_p1a = p_trigger_election
        .then(p_ballot.clone())
        .all_ticks()
        .inspect(q!(|_| println!("Proposer leader expired, sending P1a")))
        .broadcast_bincode_interleaved(acceptors);

    let (a_max_ballot, a_to_proposers_p1b) =
        acceptor_p1(acceptor_tick, p_to_acceptors_p1a, a_log, proposers);

    let (p_is_leader, p_accepted_values, fail_ballots) = p_p1b(
        proposer_tick,
        a_to_proposers_p1b.inspect(q!(|p1b| println!("Proposer received P1b: {:?}", p1b))),
        p_ballot.clone(),
        p_has_largest_ballot,
        f,
    );
    p_is_leader_complete_cycle.complete(p_is_leader.clone());
    p1b_fail_complete.complete(fail_ballots.all_ticks());

    (p_ballot, p_is_leader, p_accepted_values, a_max_ballot)
}

// Proposer logic to calculate the next ballot number. Expects p_received_max_ballot, the largest ballot received so far. Outputs streams: ballot_num, and has_largest_ballot, which only contains a value if we have the largest ballot.
#[expect(clippy::type_complexity, reason = "internal paxos code // TODO")]
fn p_ballot_calc<'a>(
    proposer_tick: &Tick<Cluster<'a, Proposer>>,
    p_received_max_ballot: Singleton<Ballot, Tick<Cluster<'a, Proposer>>, Bounded>,
) -> (
    Singleton<Ballot, Tick<Cluster<'a, Proposer>>, Bounded>,
    Optional<(), Tick<Cluster<'a, Proposer>>, Bounded>,
) {
    let (p_ballot_num_complete_cycle, p_ballot_num) =
        proposer_tick.cycle_with_initial(proposer_tick.singleton(q!(0)));

    let p_new_ballot_num = p_received_max_ballot
        .clone()
        .zip(p_ballot_num.clone())
        .map(q!(move |(received_max_ballot, ballot_num)| {
            if received_max_ballot
                > (Ballot {
                    num: ballot_num,
                    proposer_id: CLUSTER_SELF_ID,
                })
            {
                received_max_ballot.num + 1
            } else {
                ballot_num
            }
        }));
    p_ballot_num_complete_cycle.complete_next_tick(p_new_ballot_num);

    let p_ballot = p_ballot_num.map(q!(move |num| Ballot {
        num,
        proposer_id: CLUSTER_SELF_ID
    }));

    let p_has_largest_ballot = p_received_max_ballot
        .clone()
        .zip(p_ballot.clone())
        .filter(q!(
            |(received_max_ballot, cur_ballot)| *received_max_ballot <= *cur_ballot
        ))
        .map(q!(|_| ()));

    // End stable leader election
    (p_ballot, p_has_largest_ballot)
}

fn p_leader_expired<'a>(
    proposer_tick: &Tick<Cluster<'a, Proposer>>,
    p_to_proposers_i_am_leader: Stream<Ballot, Cluster<'a, Proposer>, Unbounded, NoOrder>,
    p_is_leader: Optional<(), Tick<Cluster<'a, Proposer>>, Bounded>,
    i_am_leader_check_timeout: u64, // How often to check if heartbeat expired
) -> Optional<Option<Instant>, Tick<Cluster<'a, Proposer>>, Bounded> {
    let p_latest_received_i_am_leader = p_to_proposers_i_am_leader.clone().fold_commutative(
        q!(|| None),
        q!(|latest, _| {
            // Note: May want to check received ballot against our own?
            *latest = Some(Instant::now());
        }),
    );

    p_latest_received_i_am_leader
        .latest_tick(proposer_tick)
        .continue_unless(p_is_leader)
        .filter(q!(move |latest_received_i_am_leader| {
            if let Some(latest_received_i_am_leader) = latest_received_i_am_leader {
                (Instant::now().duration_since(*latest_received_i_am_leader))
                    > Duration::from_secs(i_am_leader_check_timeout)
            } else {
                true
            }
        }))
}

#[expect(clippy::type_complexity, reason = "internal paxos code // TODO")]
fn p_leader_heartbeat<'a>(
    proposers: &Cluster<'a, Proposer>,
    proposer_tick: &Tick<Cluster<'a, Proposer>>,
    p_is_leader: Optional<(), Tick<Cluster<'a, Proposer>>, Bounded>,
    p_ballot: Singleton<Ballot, Tick<Cluster<'a, Proposer>>, Bounded>,
    i_am_leader_send_timeout: u64,  // How often to heartbeat
    i_am_leader_check_timeout: u64, // How often to check if heartbeat expired
    i_am_leader_check_timeout_delay_multiplier: usize, /* Initial delay, multiplied by proposer pid, to stagger proposers checking for timeouts */
) -> (
    Stream<Ballot, Cluster<'a, Proposer>, Unbounded, NoOrder>,
    Optional<Option<Instant>, Tick<Cluster<'a, Proposer>>, Bounded>,
) {
    let p_to_proposers_i_am_leader = p_is_leader
        .clone()
        .then(p_ballot)
        .latest()
        .sample_every(q!(Duration::from_secs(i_am_leader_send_timeout)))
        .broadcast_bincode_interleaved(proposers);

    let p_leader_expired = p_leader_expired(
        proposer_tick,
        p_to_proposers_i_am_leader.clone(),
        p_is_leader,
        i_am_leader_check_timeout,
    );

    // Add random delay depending on node ID so not everyone sends p1a at the same time
    let p_trigger_election = p_leader_expired.continue_if(
        proposers
            .source_interval_delayed(
                q!(Duration::from_secs(
                    (CLUSTER_SELF_ID.raw_id * i_am_leader_check_timeout_delay_multiplier as u32)
                        .into()
                )),
                q!(Duration::from_secs(i_am_leader_check_timeout)),
            )
            .tick_batch(proposer_tick)
            .first(),
    );
    (p_to_proposers_i_am_leader, p_trigger_election)
}

#[expect(clippy::type_complexity, reason = "internal paxos code // TODO")]
fn acceptor_p1<'a, L: Serialize + DeserializeOwned + Clone>(
    acceptor_tick: &Tick<Cluster<'a, Acceptor>>,
    p_to_acceptors_p1a: Stream<Ballot, Cluster<'a, Acceptor>, Unbounded, NoOrder>,
    a_log: Singleton<L, Tick<Cluster<'a, Acceptor>>, Bounded>,
    proposers: &Cluster<'a, Proposer>,
) -> (
    Singleton<Ballot, Tick<Cluster<'a, Acceptor>>, Bounded>,
    Stream<(Ballot, Result<L, Ballot>), Cluster<'a, Proposer>, Unbounded, NoOrder>,
) {
    let p_to_acceptors_p1a = p_to_acceptors_p1a.tick_batch(acceptor_tick);
    let a_max_ballot = p_to_acceptors_p1a
        .clone()
        .inspect(q!(|p1a| println!("Acceptor received P1a: {:?}", p1a)))
        .persist()
        .max()
        .unwrap_or(acceptor_tick.singleton(q!(Ballot {
            num: 0,
            proposer_id: ClusterId::from_raw(0)
        })));

    (
        a_max_ballot.clone(),
        p_to_acceptors_p1a
            .cross_singleton(a_max_ballot)
            .cross_singleton(a_log)
            .map(q!(|((ballot, max_ballot), log)| (
                ballot.proposer_id,
                (
                    ballot,
                    if ballot == max_ballot {
                        Ok(log)
                    } else {
                        Err(max_ballot)
                    }
                )
            )))
            .all_ticks()
            .send_bincode_interleaved(proposers),
    )
}

// Proposer logic for processing p1bs, determining if the proposer is now the leader, which uncommitted messages to commit, what the maximum slot is in the p1bs, and which no-ops to commit to fill log holes.
#[expect(clippy::type_complexity, reason = "internal paxos code // TODO")]
fn p_p1b<'a, P: Clone + Serialize + DeserializeOwned>(
    proposer_tick: &Tick<Cluster<'a, Proposer>>,
    a_to_proposers_p1b: Stream<
        (Ballot, Result<P, Ballot>),
        Cluster<'a, Proposer>,
        Unbounded,
        NoOrder,
    >,
    p_ballot: Singleton<Ballot, Tick<Cluster<'a, Proposer>>, Bounded>,
    p_has_largest_ballot: Optional<(), Tick<Cluster<'a, Proposer>>, Bounded>,
    f: usize,
) -> (
    Optional<(), Tick<Cluster<'a, Proposer>>, Bounded>,
    Stream<P, Tick<Cluster<'a, Proposer>>, Bounded, NoOrder>,
    Stream<Ballot, Tick<Cluster<'a, Proposer>>, Bounded, NoOrder>,
) {
    let (quorums, fails) = collect_quorum(
        proposer_tick,
        a_to_proposers_p1b.tick_batch(proposer_tick),
        f + 1,
        2 * f + 1,
    );

    let p_received_quorum_of_p1bs = quorums
        .persist()
        .fold_keyed_commutative(
            q!(|| vec![]),
            q!(|logs, log| {
                logs.push(log);
            }),
        )
        .max_by_key(q!(|t| t.0))
        .zip(p_ballot.clone())
        .filter_map(q!(
            move |((quorum_ballot, quorum_accepted), my_ballot)| if quorum_ballot == my_ballot {
                Some(quorum_accepted)
            } else {
                None
            }
        ));

    let p_is_leader = p_received_quorum_of_p1bs
        .clone()
        .map(q!(|_| ()))
        .continue_if(p_has_largest_ballot.clone());

    (
        p_is_leader,
        // The fold was not commutative, so this is unordered.
        p_received_quorum_of_p1bs.flatten().assume_ordering(),
        fails.map(q!(|(_, ballot)| ballot)),
    )
}

#[expect(clippy::type_complexity, reason = "internal paxos code // TODO")]
fn recommit_after_leader_election<'a, P: PaxosPayload>(
    accepted_logs: Stream<
        HashMap<usize, LogValue<P>>,
        Tick<Cluster<'a, Proposer>>,
        Bounded,
        NoOrder,
    >,
    p_ballot: Singleton<Ballot, Tick<Cluster<'a, Proposer>>, Bounded>,
    f: usize,
) -> (
    Stream<P2a<P>, Tick<Cluster<'a, Proposer>>, Bounded, NoOrder>,
    Optional<usize, Tick<Cluster<'a, Proposer>>, Bounded>,
) {
    let p_p1b_highest_entries_and_count = accepted_logs
        .flatten() // Convert HashMap log back to stream
        .fold_keyed_commutative::<(usize, Option<LogValue<P>>), _, _>(q!(|| (0, None)), q!(|curr_entry, new_entry| {
            if let Some(curr_entry_payload) = &mut curr_entry.1 {
                let same_values = new_entry.value == curr_entry_payload.value;
                let higher_ballot = new_entry.ballot > curr_entry_payload.ballot;
                // Increment count if the values are the same
                if same_values {
                    curr_entry.0 += 1;
                }
                // Replace the ballot with the largest one
                if higher_ballot {
                    curr_entry_payload.ballot = new_entry.ballot;
                    // Replace the value with the one from the largest ballot, if necessary
                    if !same_values {
                        curr_entry.0 = 1;
                        curr_entry_payload.value = new_entry.value;
                    }
                }
            } else {
                *curr_entry = (1, Some(new_entry));
            }
        }));
    let p_log_to_try_commit = p_p1b_highest_entries_and_count
        .clone()
        .cross_singleton(p_ballot.clone())
        .filter_map(q!(move |((slot, (count, entry)), ballot)| {
            let entry = entry.unwrap();
            if count <= f {
                Some(P2a {
                    ballot,
                    slot,
                    value: entry.value,
                })
            } else {
                None
            }
        }));
    let p_max_slot = p_p1b_highest_entries_and_count
        .clone()
        .map(q!(|(slot, _)| slot))
        .max();
    let p_proposed_slots = p_p1b_highest_entries_and_count
        .clone()
        .map(q!(|(slot, _)| slot));
    let p_log_holes = p_max_slot
        .clone()
        .flat_map(q!(|max_slot| 0..max_slot))
        .filter_not_in(p_proposed_slots)
        .cross_singleton(p_ballot.clone())
        .map(q!(|(slot, ballot)| P2a {
            ballot,
            slot,
            value: None
        }));

    (p_log_to_try_commit.union(p_log_holes), p_max_slot)
}

#[expect(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "internal paxos code // TODO"
)]
fn sequence_payload<'a, P: PaxosPayload, R>(
    proposers: &Cluster<'a, Proposer>,
    acceptors: &Cluster<'a, Acceptor>,
    proposer_tick: &Tick<Cluster<'a, Proposer>>,
    acceptor_tick: &Tick<Cluster<'a, Acceptor>>,
    c_to_proposers: Stream<P, Cluster<'a, Proposer>, Unbounded>,
    r_to_acceptors_checkpoint: Stream<
        (ClusterId<R>, usize),
        Cluster<'a, Acceptor>,
        Unbounded,
        NoOrder,
    >,

    p_ballot: Singleton<Ballot, Tick<Cluster<'a, Proposer>>, Bounded>,
    p_is_leader: Optional<(), Tick<Cluster<'a, Proposer>>, Bounded>,

    p_relevant_p1bs: Stream<
        HashMap<usize, LogValue<P>>,
        Tick<Cluster<'a, Proposer>>,
        Bounded,
        NoOrder,
    >,
    f: usize,

    a_max_ballot: Singleton<Ballot, Tick<Cluster<'a, Acceptor>>, Bounded>,
) -> (
    Stream<(usize, Option<P>), Cluster<'a, Proposer>, Unbounded, NoOrder>,
    Singleton<HashMap<usize, LogValue<P>>, Tick<Cluster<'a, Acceptor>>, Bounded>,
    Stream<Ballot, Cluster<'a, Proposer>, Unbounded, NoOrder>,
) {
    let (p_log_to_recommit, p_max_slot) =
        recommit_after_leader_election(p_relevant_p1bs, p_ballot.clone(), f);

    let indexed_payloads = index_payloads(
        proposer_tick,
        p_max_slot,
        c_to_proposers,
        p_is_leader.clone(),
    );

    let payloads_to_send = indexed_payloads
        .cross_singleton(p_ballot.clone())
        .map(q!(|((slot, payload), ballot)| (
            (slot, ballot),
            Some(payload)
        )))
        .union(p_log_to_recommit.map(q!(|p2a| ((p2a.slot, p2a.ballot), p2a.value))))
        .continue_if(p_is_leader);

    let (a_log, a_to_proposers_p2b) = acceptor_p2(
        acceptor_tick,
        a_max_ballot.clone(),
        payloads_to_send
            .clone()
            .all_ticks()
            .map(q!(|((slot, ballot), value)| P2a {
                ballot,
                slot,
                value
            }))
            .broadcast_bincode_interleaved(acceptors),
        r_to_acceptors_checkpoint,
        proposers,
        f,
    );

    // TOOD: only persist if we are the leader
    let (quorums, fails) = collect_quorum(
        proposer_tick,
        a_to_proposers_p2b.clone().tick_batch(proposer_tick),
        f + 1,
        2 * f + 1,
    );

    let p_to_replicas = join_responses(
        proposer_tick,
        quorums.keys().map(q!(|k| (k, ()))),
        payloads_to_send,
    )
    .all_ticks();

    (
        p_to_replicas.map(q!(|((slot, _ballot), (value, _))| (slot, value))),
        a_log.map(q!(|(_ckpnt, log)| log)),
        fails.map(q!(|(_, ballot)| ballot)).all_ticks(),
    )
}

#[derive(Clone)]
enum CheckpointOrP2a<P> {
    Checkpoint(usize),
    P2a(P2a<P>),
}

// Proposer logic to send p2as, outputting the next slot and the p2as to send to acceptors.
fn index_payloads<'a, P: PaxosPayload>(
    proposer_tick: &Tick<Cluster<'a, Proposer>>,
    p_max_slot: Optional<usize, Tick<Cluster<'a, Proposer>>, Bounded>,
    c_to_proposers: Stream<P, Cluster<'a, Proposer>, Unbounded>,
    p_is_leader: Optional<(), Tick<Cluster<'a, Proposer>>, Bounded>,
) -> Stream<(usize, P), Tick<Cluster<'a, Proposer>>, Bounded> {
    let (p_next_slot_complete_cycle, p_next_slot) =
        proposer_tick.cycle_with_initial::<Singleton<usize, _, _>>(proposer_tick.singleton(q!(0)));
    let p_next_slot_after_reconciling_p1bs = p_max_slot.map(q!(|max_slot| max_slot + 1));

    let base_slot = p_next_slot_after_reconciling_p1bs.unwrap_or(p_next_slot);

    let p_indexed_payloads = c_to_proposers
        .tick_batch(proposer_tick)
        .continue_if(p_is_leader)
        .enumerate()
        .cross_singleton(base_slot.clone())
        .map(q!(|((index, payload), base_slot)| (
            base_slot + index,
            payload
        )));

    let p_num_payloads = p_indexed_payloads.clone().count();
    let p_next_slot_after_sending_payloads =
        p_num_payloads
            .clone()
            .zip(base_slot)
            .map(q!(|(num_payloads, base_slot)| base_slot + num_payloads));

    p_next_slot_complete_cycle.complete_next_tick(p_next_slot_after_sending_payloads);
    p_indexed_payloads
}

#[expect(clippy::type_complexity, reason = "internal paxos code // TODO")]
fn acceptor_p2<'a, P: PaxosPayload, R>(
    acceptor_tick: &Tick<Cluster<'a, Acceptor>>,
    a_max_ballot: Singleton<Ballot, Tick<Cluster<'a, Acceptor>>, Bounded>,
    p_to_acceptors_p2a: Stream<P2a<P>, Cluster<'a, Acceptor>, Unbounded, NoOrder>,
    r_to_acceptors_checkpoint: Stream<
        (ClusterId<R>, usize),
        Cluster<'a, Acceptor>,
        Unbounded,
        NoOrder,
    >,
    proposers: &Cluster<'a, Proposer>,
    f: usize,
) -> (
    Singleton<(Option<usize>, HashMap<usize, LogValue<P>>), Tick<Cluster<'a, Acceptor>>, Bounded>,
    Stream<((usize, Ballot), Result<(), Ballot>), Cluster<'a, Proposer>, Unbounded, NoOrder>,
) {
    let p_to_acceptors_p2a_batch = p_to_acceptors_p2a.tick_batch(acceptor_tick);

    // Get the latest checkpoint sequence per replica
    let a_checkpoint_largest_seqs = r_to_acceptors_checkpoint
        .tick_prefix(acceptor_tick)
        .reduce_keyed_commutative(q!(|curr_seq, seq| {
            if seq > *curr_seq {
                *curr_seq = seq;
            }
        }));
    let a_checkpoints_quorum_reached = a_checkpoint_largest_seqs.clone().count().filter_map(q!(
        move |num_received| if num_received == f + 1 {
            Some(true)
        } else {
            None
        }
    ));
    // Find the smallest checkpoint seq that everyone agrees to, track whenever it changes
    let a_new_checkpoint = a_checkpoint_largest_seqs
        .continue_if(a_checkpoints_quorum_reached)
        .map(q!(|(_sender, seq)| seq))
        .min()
        .delta()
        .map(q!(|min_seq| CheckpointOrP2a::Checkpoint(min_seq)));
    // .inspect(q!(|(min_seq, p2a): &(i32, P2a)| println!("Acceptor new checkpoint: {:?}", min_seq)));

    let a_p2as_to_place_in_log = p_to_acceptors_p2a_batch
        .clone()
        .cross_singleton(a_max_ballot.clone()) // Don't consider p2as if the current ballot is higher
        .filter_map(q!(|(p2a, max_ballot)|
            if p2a.ballot >= max_ballot {
                Some(CheckpointOrP2a::P2a(p2a))
            } else {
                None
            }
        ));
    let a_log = a_p2as_to_place_in_log
        .union(a_new_checkpoint.into_stream())
        .persist()
        .fold_commutative(
            q!(|| (None, HashMap::new())),
            q!(|(prev_checkpoint, log), checkpoint_or_p2a| {
                match checkpoint_or_p2a {
                    CheckpointOrP2a::Checkpoint(new_checkpoint) => {
                        if prev_checkpoint
                            .map(|prev| new_checkpoint > prev)
                            .unwrap_or(true)
                        {
                            for slot in (prev_checkpoint.unwrap_or(0))..new_checkpoint {
                                log.remove(&slot);
                            }

                            *prev_checkpoint = Some(new_checkpoint);
                        }
                    }
                    CheckpointOrP2a::P2a(p2a) => {
                        // This is a regular p2a message. Insert it into the log if it is not checkpointed and has a higher ballot than what was there before
                        if prev_checkpoint.map(|prev| p2a.slot > prev).unwrap_or(true)
                            && log
                                .get(&p2a.slot)
                                .map(|prev_p2a: &LogValue<_>| p2a.ballot > prev_p2a.ballot)
                                .unwrap_or(true)
                        {
                            log.insert(
                                p2a.slot,
                                LogValue {
                                    ballot: p2a.ballot,
                                    value: p2a.value,
                                },
                            );
                        }
                    }
                }
            }),
        );

    let a_to_proposers_p2b = p_to_acceptors_p2a_batch
        .cross_singleton(a_max_ballot)
        .map(q!(|(p2a, max_ballot)| (
            p2a.ballot.proposer_id,
            (
                (p2a.slot, p2a.ballot),
                if p2a.ballot == max_ballot {
                    Ok(())
                } else {
                    Err(max_ballot)
                }
            )
        )))
        .all_ticks()
        .send_bincode_interleaved(proposers);
    (a_log, a_to_proposers_p2b)
}
