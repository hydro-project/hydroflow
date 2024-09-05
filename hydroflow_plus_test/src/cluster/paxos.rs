use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use hydroflow_plus::*;
use serde::{Deserialize, Serialize};
use stageleft::*;
use tokio::time::Instant;

pub struct Proposer {}
pub struct Acceptor {}
pub struct Client {}
pub struct Replica {}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
struct ClientPayload {
    key: u32,
    value: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
struct ReplicaPayload {
    // Note: Important that seq is the first member of the struct for sorting
    seq: i32,
    key: u32,
    value: String,
}

pub trait Address {
    fn get_id(&self) -> u32;
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash)]
pub struct Ballot {
    // Note: Important that num comes before id, since Ord is defined lexicographically
    pub num: u32,
    pub id: u32,
}

impl Address for Ballot {
    fn get_id(&self) -> u32 {
        self.id
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct P1a {
    ballot: Ballot,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LogValue {
    ballot: Ballot,
    value: ClientPayload,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct P1b {
    ballot: Ballot,
    max_ballot: Ballot,
    accepted: HashMap<i32, LogValue>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
struct P2a {
    ballot: Ballot,
    slot: i32,
    value: ClientPayload,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
struct P2b {
    ballot: Ballot,
    max_ballot: Ballot,
    slot: i32,
    value: ClientPayload,
}

// Important: By convention, all relations that represent booleans either have a single "true" value or nothing.
// This allows us to use the continue_if_exists() and continue_if_empty() operators as if they were if (true) and if (false) statements.
#[allow(clippy::too_many_arguments)]
pub fn paxos(
    flow: &FlowBuilder<'_>,
    f: usize,
    num_clients_per_node: usize,
    median_latency_window_size: usize, /* How many latencies to keep in the window for calculating the median */
    checkpoint_frequency: usize,       // How many sequence numbers to commit before checkpointing
    i_am_leader_send_timeout: u64,     // How often to heartbeat
    i_am_leader_check_timeout: u64,    // How often to check if heartbeat expired
    i_am_leader_check_timeout_delay_multiplier: usize, /* Initial delay, multiplied by proposer pid, to stagger proposers checking for timeouts */
) -> (
    Cluster<Proposer>,
    Cluster<Acceptor>,
    Cluster<Client>,
    Cluster<Replica>,
) {
    let proposers = flow.cluster::<Proposer>();
    let acceptors = flow.cluster::<Acceptor>();
    let clients = flow.cluster::<Client>();
    let replicas = flow.cluster::<Replica>();

    let (r_to_clients_payload_applied_cycle, r_to_clients_payload_applied) = flow.cycle(&clients);
    let (p_to_clients_leader_elected_cycle, p_to_clients_leader_elected) = flow.cycle(&clients);

    let c_to_proposers = client(
        &clients,
        p_to_clients_leader_elected,
        r_to_clients_payload_applied,
        flow,
        num_clients_per_node,
        median_latency_window_size,
        f,
    )
    .send_bincode_interleaved(&proposers);

    // Proposers.
    flow.source_iter(&proposers, q!(["Proposers say hello"]))
        .for_each(q!(|s| println!("{}", s)));
    let p_id = flow.cluster_self_id(&proposers);

    let (p_to_proposers_i_am_leader_complete_cycle, p_to_proposers_i_am_leader) =
        flow.cycle::<Stream<Ballot, Unbounded, NoTick, Cluster<Proposer>>>(&proposers);
    let (a_to_proposers_p1b_complete_cycle, a_to_proposers_p1b) =
        flow.cycle::<Stream<(u32, P1b), _, _, _>>(&proposers);
    a_to_proposers_p1b
        .clone()
        .for_each(q!(|(_, p1b)| println!("Proposer received P1b: {:?}", p1b)));
    let (a_to_proposers_p2b_complete_cycle, a_to_proposers_p2b) =
        flow.cycle::<Stream<(u32, P2b), _, _, _>>(&proposers);
    // a_to_proposers_p2b.clone().for_each(q!(|(_, p2b): (u32, P2b)| println!("Proposer received P2b: {:?}", p2b)));
    // p_to_proposers_i_am_leader.clone().for_each(q!(|ballot: Ballot| println!("Proposer received I am leader: {:?}", ballot)));
    // c_to_proposers.clone().for_each(q!(|payload: ClientPayload| println!("Client sent proposer payload: {:?}", payload)));

    let p_received_max_ballot = p_max_ballot(
        flow,
        &proposers,
        a_to_proposers_p1b.clone(),
        a_to_proposers_p2b.clone(),
        p_to_proposers_i_am_leader.clone(),
    );
    let (p_ballot_num, p_has_largest_ballot) =
        p_ballot_calc(flow, &proposers, p_received_max_ballot.latest_tick());

    let (p_is_leader, p_log_to_try_commit, p_max_slot, p_log_holes) = p_p1b(
        flow,
        &proposers,
        a_to_proposers_p1b,
        p_ballot_num.clone(),
        p_has_largest_ballot,
        f,
    );

    let (p_to_proposers_i_am_leader_from_others, p_to_acceptors_p1a) = p_p1a(
        p_ballot_num.clone(),
        p_is_leader.clone(),
        &proposers,
        p_to_proposers_i_am_leader,
        flow,
        &acceptors,
        i_am_leader_send_timeout,
        i_am_leader_check_timeout,
        i_am_leader_check_timeout_delay_multiplier,
    );
    p_to_proposers_i_am_leader_complete_cycle.complete(p_to_proposers_i_am_leader_from_others);

    let (p_next_slot, p_to_acceptors_p2a) = p_p2a(
        flow,
        &proposers,
        p_max_slot,
        c_to_proposers,
        p_ballot_num.clone(),
        p_log_to_try_commit,
        p_log_holes,
        p_is_leader.clone(),
        &acceptors,
    );

    // Tell clients that leader election has completed and they can begin sending messages
    let p_to_clients_new_leader_elected = p_is_leader.clone()
        .continue_unless(p_next_slot)
        .cross_singleton(p_ballot_num)
        .map(q!(move |(_is_leader, ballot_num)| Ballot { num: ballot_num, id: p_id})) // Only tell the clients once when leader election concludes
        .all_ticks()
        .broadcast_bincode_interleaved(&clients);
    p_to_clients_leader_elected_cycle.complete(p_to_clients_new_leader_elected);
    // End tell clients that leader election has completed
    let p_to_replicas = p_p2b(flow, &proposers, a_to_proposers_p2b, &replicas, f);

    // Acceptors.
    flow.source_iter(&acceptors, q!(["Acceptors say hello"]))
        .for_each(q!(|s| println!("{}", s)));
    let (r_to_acceptors_checkpoint_complete_cycle, r_to_acceptors_checkpoint) =
        flow.cycle(&acceptors);
    p_to_acceptors_p1a
        .clone()
        .for_each(q!(|p1a| println!("Acceptor received P1a: {:?}", p1a)));
    // p_to_acceptors_p2a.clone().for_each(q!(|p2a: P2a| println!("Acceptor received P2a: {:?}", p2a)));
    let (a_to_proposers_p1b_new, a_to_proposers_p2b_new) = acceptor(
        flow,
        p_to_acceptors_p1a,
        p_to_acceptors_p2a,
        r_to_acceptors_checkpoint,
        &proposers,
        &acceptors,
        f,
    );
    a_to_proposers_p1b_complete_cycle.complete(a_to_proposers_p1b_new);
    a_to_proposers_p2b_complete_cycle.complete(a_to_proposers_p2b_new);

    // Replicas.
    let (r_to_acceptors_checkpoint_new, r_new_processed_payloads) =
        replica(flow, &replicas, p_to_replicas, checkpoint_frequency);
    r_to_clients_payload_applied_cycle.complete(r_new_processed_payloads.send_bincode(&clients));
    r_to_acceptors_checkpoint_complete_cycle
        .complete(r_to_acceptors_checkpoint_new.broadcast_bincode(&acceptors));

    (proposers, acceptors, clients, replicas)
}

#[allow(clippy::type_complexity)]
fn acceptor<'a>(
    flow: &FlowBuilder<'a>,
    p_to_acceptors_p1a: Stream<'a, P1a, Unbounded, NoTick, Cluster<Acceptor>>,
    p_to_acceptors_p2a: Stream<'a, P2a, Unbounded, NoTick, Cluster<Acceptor>>,
    r_to_acceptors_checkpoint: Stream<'a, (u32, i32), Unbounded, NoTick, Cluster<Acceptor>>,
    proposers: &Cluster<Proposer>,
    acceptors: &Cluster<Acceptor>,
    f: usize,
) -> (
    Stream<'a, (u32, P1b), Unbounded, NoTick, Cluster<Proposer>>,
    Stream<'a, (u32, P2b), Unbounded, NoTick, Cluster<Proposer>>,
) {
    // Get the latest checkpoint sequence per replica
    let a_checkpoint_largest_seqs =
        r_to_acceptors_checkpoint
            .tick_prefix()
            .reduce_keyed(q!(|curr_seq, seq| {
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
        .unwrap_or(flow.singleton(acceptors, q!(-1)).latest_tick())
        .delta()
        .map(q!(|min_seq| (
            min_seq,
            P2a {
                // Create tuple with checkpoint number and dummy p2a
                ballot: Ballot { num: 0, id: 0 },
                slot: -1,
                value: ClientPayload {
                    key: 0,
                    value: "".to_string(),
                }
            }
        )));
    // .inspect(q!(|(min_seq, p2a): &(i32, P2a)| println!("Acceptor new checkpoint: {:?}", min_seq)));

    let a_max_ballot = p_to_acceptors_p1a
        .clone()
        .map(q!(|p1a| p1a.ballot))
        .max()
        .unwrap_or(flow.singleton(acceptors, q!(Ballot { num: 0, id: 0 })));
    let a_p2as_to_place_in_log = p_to_acceptors_p2a
        .clone()
        .tick_batch()
        .cross_singleton(a_max_ballot.clone().latest_tick()) // Don't consider p2as if the current ballot is higher
        .filter_map(q!(|(p2a, max_ballot)|
            if p2a.ballot >= max_ballot {
                Some((-1, p2a)) // Signal that this isn't a checkpoint with -1
            } else {
                None
            }
        ));
    let a_log = a_p2as_to_place_in_log
        .union(a_new_checkpoint.into_stream())
        .all_ticks()
        .fold(
            q!(|| (-1, HashMap::<i32, LogValue>::new())),
            q!(|(prev_checkpoint, log), (new_checkpoint, p2a)| {
                if new_checkpoint != -1 {
                    // This is a checkpoint message. Delete all entries up to the checkpoint
                    for slot in *prev_checkpoint..new_checkpoint {
                        log.remove(&slot);
                    }
                    *prev_checkpoint = new_checkpoint;
                } else {
                    // This is a regular p2a message. Insert it into the log if it is not checkpointed and has a higher ballot than what was there before
                    if p2a.slot > *prev_checkpoint {
                        match log.get(&p2a.slot) {
                            None => {
                                log.insert(
                                    p2a.slot,
                                    LogValue {
                                        ballot: p2a.ballot,
                                        value: p2a.value,
                                    },
                                );
                            }
                            Some(prev_p2a) => {
                                if p2a.ballot > prev_p2a.ballot {
                                    log.insert(
                                        p2a.slot,
                                        LogValue {
                                            ballot: p2a.ballot,
                                            value: p2a.value,
                                        },
                                    );
                                }
                            }
                        };
                    }
                }
            }),
        );

    let a_to_proposers_p1b_new = p_to_acceptors_p1a
        .tick_batch()
        .cross_singleton(a_max_ballot.clone().latest_tick())
        .cross_singleton(a_log.latest_tick())
        .map(q!(|((p1a, max_ballot), (_prev_checkpoint, log))| (
            p1a.ballot.id,
            P1b {
                ballot: p1a.ballot,
                max_ballot,
                accepted: log
            }
        )))
        .all_ticks()
        .send_bincode(proposers);
    let a_to_proposers_p2b_new = p_to_acceptors_p2a
        .tick_batch()
        .cross_singleton(a_max_ballot.latest_tick())
        .map(q!(|(p2a, max_ballot)| (
            p2a.ballot.id,
            P2b {
                ballot: p2a.ballot,
                max_ballot,
                slot: p2a.slot,
                value: p2a.value
            }
        )))
        .all_ticks()
        .send_bincode(proposers);
    (a_to_proposers_p1b_new, a_to_proposers_p2b_new)
}

fn p_p2b<'a>(
    flow: &FlowBuilder<'a>,
    proposers: &Cluster<Proposer>,
    a_to_proposers_p2b: Stream<'a, (u32, P2b), Unbounded, NoTick, Cluster<Proposer>>,
    replicas: &Cluster<Replica>,
    f: usize,
) -> Stream<'a, ReplicaPayload, Unbounded, NoTick, Cluster<Replica>> {
    let (p_broadcasted_p2b_slots_complete_cycle, p_broadcasted_p2b_slots) =
        flow.tick_cycle(proposers);
    let (p_persisted_p2bs_complete_cycle, p_persisted_p2bs) = flow.tick_cycle(proposers);
    let p_p2b = a_to_proposers_p2b
        .clone()
        .tick_batch()
        .union(p_persisted_p2bs);
    let p_count_matching_p2bs = p_p2b
        .clone()
        .filter_map(q!(|(sender, p2b)| if p2b.ballot == p2b.max_ballot {
            // Only consider p2bs where max ballot = ballot, which means that no one preempted us
            Some((p2b.slot, (sender, p2b)))
        } else {
            None
        }))
        .fold_keyed(
            q!(|| (
                0,
                P2b {
                    ballot: Ballot { num: 0, id: 0 },
                    max_ballot: Ballot { num: 0, id: 0 },
                    slot: 0,
                    value: ClientPayload {
                        key: 0,
                        value: "0".to_string()
                    }
                }
            )),
            q!(|accum, (_sender, p2b)| {
                accum.0 += 1;
                accum.1 = p2b;
            }),
        );
    let p_p2b_quorum_reached = p_count_matching_p2bs
        .clone()
        .filter(q!(move |(_slot, (count, _p2b))| *count > f));
    let p_to_replicas = p_p2b_quorum_reached
        .clone()
        .anti_join(p_broadcasted_p2b_slots) // Only tell the replicas about committed values once
        .map(q!(|(_slot, (_count, p2b))| ReplicaPayload { seq: p2b.slot, key: p2b.value.key, value: p2b.value.value }))
        .all_ticks()
        .broadcast_bincode_interleaved(replicas);

    let p_p2b_all_commit_slots =
        p_count_matching_p2bs
            .clone()
            .filter_map(q!(move |(slot, (count, _p2b))| if count == 2 * f + 1 {
                Some(slot)
            } else {
                None
            }));
    // p_p2b_all_commit_slots.inspect(q!(|slot: i32| println!("Proposer slot all received: {:?}", slot)));
    let p_broadcasted_p2b_slots_new = p_p2b_quorum_reached
        .clone()
        .map(q!(|(slot, (_count, _p2b))| slot))
        .filter_not_in(p_p2b_all_commit_slots.clone());
    // p_broadcasted_p2b_slots_new.inspect(q!(|slot: i32| println!("Proposer slot broadcasted: {:?}", slot)));
    p_broadcasted_p2b_slots_complete_cycle.complete_next_tick(p_broadcasted_p2b_slots_new);
    let p_persisted_p2bs_new = p_p2b
        .clone()
        .map(q!(|(sender, p2b)| (p2b.slot, (sender, p2b))))
        .anti_join(p_p2b_all_commit_slots.clone())
        .map(q!(|(_slot, (sender, p2b))| (sender, p2b)));
    // p_persisted_p2bs_new.inspect(q!(|(sender, p2b): (u32, P2b)| println!("Proposer persisting p2b: {:?}", p2b)));
    p_persisted_p2bs_complete_cycle.complete_next_tick(p_persisted_p2bs_new);
    p_to_replicas
}

// Proposer logic to send p2as, outputting the next slot and the p2as to send to acceptors.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn p_p2a<'a>(
    flow: &FlowBuilder<'a>,
    proposers: &Cluster<Proposer>,
    p_max_slot: Optional<'a, i32, Bounded, Tick, Cluster<Proposer>>,
    c_to_proposers: Stream<'a, ClientPayload, Unbounded, NoTick, Cluster<Proposer>>,
    p_ballot_num: Singleton<'a, u32, Bounded, Tick, Cluster<Proposer>>,
    p_log_to_try_commit: Stream<'a, P2a, Bounded, Tick, Cluster<Proposer>>,
    p_log_holes: Stream<'a, P2a, Bounded, Tick, Cluster<Proposer>>,
    p_is_leader: Optional<'a, bool, Bounded, Tick, Cluster<Proposer>>,
    acceptors: &Cluster<Acceptor>,
) -> (
    Optional<'a, i32, Bounded, Tick, Cluster<Proposer>>,
    Stream<'a, P2a, Unbounded, NoTick, Cluster<Acceptor>>,
) {
    let p_id = flow.cluster_self_id(proposers);
    let (p_next_slot_complete_cycle, p_next_slot) =
        flow.tick_cycle::<Optional<i32, _, _, _>>(proposers);
    let p_next_slot_after_reconciling_p1bs = p_max_slot
        .unwrap_or(flow.singleton(proposers, q!(-1)).latest_tick())
        // .inspect(q!(|max_slot| println!("{} p_max_slot: {:?}", context.current_tick(), max_slot)))
        .continue_unless(p_next_slot.clone())
        .map(q!(|max_slot| max_slot + 1));

    // Send p2as
    let p_indexed_payloads = c_to_proposers
        .clone()
        .tick_batch()
        .enumerate()
        .cross_singleton(p_next_slot.clone())
        // .inspect(q!(|next| println!("{} p_indexed_payloads next slot: {}", context.current_tick(), next))))
        .cross_singleton(p_ballot_num.clone())
        // .inspect(q!(|ballot_num| println!("{} p_indexed_payloads ballot_num: {}", context.current_tick(), ballot_num))))
        .map(q!(move |(((index, payload), next_slot), ballot_num)| P2a { ballot: Ballot { num: ballot_num, id: p_id }, slot: next_slot + index as i32, value: payload }));
    // .inspect(q!(|p2a: &P2a| println!("{} p_indexed_payloads P2a: {:?}", context.current_tick(), p2a)));
    let p_to_acceptors_p2a = p_log_to_try_commit
        .union(p_log_holes)
        .continue_unless(p_next_slot.clone()) // Only resend p1b stuff once. Once it's resent, next_slot will exist.
        .union(p_indexed_payloads)
        .continue_if(p_is_leader.clone())
        .all_ticks()
        .broadcast_bincode_interleaved(acceptors);

    let p_num_payloads = c_to_proposers.clone().tick_batch().count();
    let p_exists_payloads = p_num_payloads
        .clone()
        .filter(q!(|num_payloads| *num_payloads > 0));
    let p_next_slot_after_sending_payloads = p_num_payloads
        .continue_if(p_exists_payloads.clone())
        .clone()
        .cross_singleton(p_next_slot.clone())
        .map(q!(
            |(num_payloads, next_slot)| next_slot + num_payloads as i32
        ));
    let p_next_slot_if_no_payloads = p_next_slot.clone().continue_unless(p_exists_payloads);
    let p_new_next_slot_calculated = p_next_slot_after_reconciling_p1bs
        // .inspect(q!(|slot| println!("{} p_new_next_slot_after_reconciling_p1bs: {:?}", context.current_tick(), slot)))
        .union(p_next_slot_after_sending_payloads)
        // .inspect(q!(|slot| println!("{} p_next_slot_after_sending_payloads: {:?}", context.current_tick(), slot))))
        .union(p_next_slot_if_no_payloads)
        // .inspect(q!(|slot| println!("{} p_next_slot_if_no_payloads: {:?}", context.current_tick(), slot))))
        .continue_if(p_is_leader.clone());
    let p_new_next_slot_default = p_is_leader // Default next slot to 0 if there haven't been any payloads at all
        .clone()
        .continue_unless(p_new_next_slot_calculated.clone())
        .map(q!(|_| 0));
    // .inspect(q!(|slot| println!("{} p_new_next_slot_default: {:?}", context.current_tick(), slot)));
    let p_new_next_slot = p_new_next_slot_calculated.union(p_new_next_slot_default);
    p_next_slot_complete_cycle.complete_next_tick(p_new_next_slot);
    (p_next_slot, p_to_acceptors_p2a)
}

// Proposer logic for processing p1bs, determining if the proposer is now the leader, which uncommitted messages to commit, what the maximum slot is in the p1bs, and which no-ops to commit to fill log holes.
#[allow(clippy::type_complexity)]
fn p_p1b<'a>(
    flow: &FlowBuilder<'a>,
    proposers: &Cluster<Proposer>,
    a_to_proposers_p1b: Stream<'a, (u32, P1b), Unbounded, NoTick, Cluster<Proposer>>,
    p_ballot_num: Singleton<'a, u32, Bounded, Tick, Cluster<Proposer>>,
    p_has_largest_ballot: Optional<'a, (Ballot, u32), Bounded, Tick, Cluster<Proposer>>,
    f: usize,
) -> (
    Optional<'a, bool, Bounded, Tick, Cluster<Proposer>>,
    Stream<'a, P2a, Bounded, Tick, Cluster<Proposer>>,
    Optional<'a, i32, Bounded, Tick, Cluster<Proposer>>,
    Stream<'a, P2a, Bounded, Tick, Cluster<Proposer>>,
) {
    let p_id = flow.cluster_self_id(proposers);
    let p_relevant_p1bs = a_to_proposers_p1b
        .clone()
        .tick_prefix()
        .cross_singleton(p_ballot_num.clone())
        .filter(q!(move |((_sender, p1b), ballot_num)| p1b.ballot
            == Ballot {
                num: *ballot_num,
                id: p_id
            }));
    let p_received_quorum_of_p1bs = p_relevant_p1bs
        .clone()
        .map(q!(|((sender, _p1b), _ballot_num)| { sender }))
        .unique()
        .count()
        .filter_map(q!(move |num_received| if num_received > f {
            Some(true)
        } else {
            None
        }));
    let p_is_leader = p_received_quorum_of_p1bs.continue_if(p_has_largest_ballot.clone());

    let p_p1b_highest_entries_and_count = p_relevant_p1bs
        .clone()
        .flat_map(q!(|((_, p1b), _)| p1b.accepted.into_iter())) // Convert HashMap log back to stream
        .fold_keyed(q!(|| (0, LogValue { ballot: Ballot { num: 0, id: 0 }, value: ClientPayload { key: 0, value: "".to_string() } })), q!(|curr_entry, new_entry| {
            let same_values = new_entry.value == curr_entry.1.value;
            let higher_ballot = new_entry.ballot > curr_entry.1.ballot;
            // Increment count if the values are the same
            if same_values {
                curr_entry.0 += 1;
            }
            // Replace the ballot with the largest one
            if higher_ballot {
                curr_entry.1.ballot = new_entry.ballot;
                // Replace the value with the one from the largest ballot, if necessary
                if !same_values {
                    curr_entry.0 = 1;
                    curr_entry.1.value = new_entry.value;
                }
            }
        }));
    let p_log_to_try_commit = p_p1b_highest_entries_and_count
        .clone()
        .cross_singleton(p_ballot_num.clone())
        .filter_map(q!(
            move |((slot, (count, entry)), ballot_num)| if count <= f as u32 {
                Some(P2a {
                    ballot: Ballot {
                        num: ballot_num,
                        id: p_id,
                    },
                    slot,
                    value: entry.value,
                })
            } else {
                None
            }
        ));
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
        .cross_singleton(p_ballot_num.clone())
        .map(q!(move |(slot, ballot_num)| P2a {
            ballot: Ballot {
                num: ballot_num,
                id: p_id
            },
            slot,
            value: ClientPayload {
                key: 0,
                value: "0".to_string()
            }
        }));
    (p_is_leader, p_log_to_try_commit, p_max_slot, p_log_holes)
}

// Replicas. All relations for replicas will be prefixed with r. Expects ReplicaPayload on p_to_replicas, outputs a stream of (client address, ReplicaPayload) after processing.
#[allow(clippy::type_complexity)]
fn replica<'a>(
    flow: &FlowBuilder<'a>,
    replicas: &Cluster<Replica>,
    p_to_replicas: Stream<'a, ReplicaPayload, Unbounded, NoTick, Cluster<Replica>>,
    checkpoint_frequency: usize,
) -> (
    Stream<'a, i32, Unbounded, NoTick, Cluster<Replica>>,
    Stream<'a, (u32, ReplicaPayload), Unbounded, NoTick, Cluster<Replica>>,
) {
    let (r_buffered_payloads_complete_cycle, r_buffered_payloads) = flow.tick_cycle(replicas);
    // p_to_replicas.inspect(q!(|payload: ReplicaPayload| println!("Replica received payload: {:?}", payload)));
    let r_sorted_payloads = p_to_replicas
        .clone()
        .tick_batch()
        .union(r_buffered_payloads) // Combine with all payloads that we've received and not processed yet
        .sort();
    // Create a cycle since we'll use this seq before we define it
    let (r_highest_seq_complete_cycle, r_highest_seq) =
        flow.tick_cycle::<Optional<'a, i32, _, _, _>>(replicas);
    let empty_slot = flow.singleton_first_tick(replicas, q!(-1));
    // Either the max sequence number executed so far or -1. Need to union otherwise r_highest_seq is empty and joins with it will fail
    let r_highest_seq_with_default = r_highest_seq.union(empty_slot);
    // Find highest the sequence number of any payload that can be processed in this tick. This is the payload right before a hole.
    let r_highest_seq_processable_payload = r_sorted_payloads
        .clone()
        .cross_singleton(r_highest_seq_with_default)
        .fold(
            q!(|| -1),
            q!(|filled_slot, (sorted_payload, highest_seq)| {
                // Note: This function only works if the input is sorted on seq.
                let next_slot = std::cmp::max(*filled_slot, highest_seq);

                *filled_slot = if sorted_payload.seq == next_slot + 1 {
                    sorted_payload.seq
                } else {
                    *filled_slot
                };
            }),
        );
    // Find all payloads that can and cannot be processed in this tick.
    let r_processable_payloads = r_sorted_payloads
        .clone()
        .cross_singleton(r_highest_seq_processable_payload.clone())
        .filter(q!(
            |(sorted_payload, highest_seq)| sorted_payload.seq <= *highest_seq
        ))
        .map(q!(|(sorted_payload, _)| { sorted_payload }));
    let r_new_non_processable_payloads = r_sorted_payloads
        .clone()
        .cross_singleton(r_highest_seq_processable_payload.clone())
        .filter(q!(
            |(sorted_payload, highest_seq)| sorted_payload.seq > *highest_seq
        ))
        .map(q!(|(sorted_payload, _)| { sorted_payload }));
    // Save these, we can process them once the hole has been filled
    r_buffered_payloads_complete_cycle.complete_next_tick(r_new_non_processable_payloads);

    let r_kv_store = r_processable_payloads
        .clone()
        .persist() // Optimization: all_ticks() + fold() = fold<static>, where the state of the previous fold is saved and persisted values are deleted.
        .fold(q!(|| (HashMap::<u32, String>::new(), -1)), q!(|state, payload| {
            let kv_store = &mut state.0;
            let last_seq = &mut state.1;
            kv_store.insert(payload.key, payload.value);
            debug_assert!(payload.seq == *last_seq + 1, "Hole in log between seq {} and {}", *last_seq, payload.seq);
            *last_seq = payload.seq;
            // println!("Replica kv store: {:?}", kv_store);
        }));
    // Update the highest seq for the next tick
    let r_new_highest_seq = r_kv_store.map(q!(|(_kv_store, highest_seq)| highest_seq));
    r_highest_seq_complete_cycle.complete_next_tick(r_new_highest_seq.clone().into());

    // Send checkpoints to the acceptors when we've processed enough payloads
    let (r_checkpointed_seqs_complete_cycle, r_checkpointed_seqs) =
        flow.tick_cycle::<Optional<'a, i32, _, _, _>>(replicas);
    let r_max_checkpointed_seq = r_checkpointed_seqs
        .persist()
        .max()
        .unwrap_or(flow.singleton(replicas, q!(-1)).latest_tick());
    let r_checkpoint_seq_new = r_max_checkpointed_seq
        .cross_singleton(r_new_highest_seq)
        .filter_map(q!(
            move |(max_checkpointed_seq, new_highest_seq)| if new_highest_seq - max_checkpointed_seq
                >= checkpoint_frequency as i32
            {
                Some(new_highest_seq)
            } else {
                None
            }
        ));
    r_checkpointed_seqs_complete_cycle.complete_next_tick(r_checkpoint_seq_new.clone());

    // Tell clients that the payload has been committed. All ReplicaPayloads contain the client's machine ID (to string) as value.
    let r_to_clients = p_to_replicas.map(q!(|payload| (
        payload.value.parse::<u32>().unwrap(),
        payload
    )));
    (r_checkpoint_seq_new.all_ticks(), r_to_clients)
}

// Clients. All relations for clients will be prefixed with c. All ClientPayloads will contain the virtual client number as key and the client's machine ID (to string) as value. Expects p_to_clients_leader_elected containing Ballots whenever the leader is elected, and r_to_clients_payload_applied containing ReplicaPayloads whenever a payload is committed. Outputs (leader address, ClientPayload) when a new leader is elected or when the previous payload is committed.
fn client<'a, B: Address + Ord + std::fmt::Debug + Clone>(
    clients: &Cluster<Client>,
    p_to_clients_leader_elected: Stream<'a, B, Unbounded, NoTick, Cluster<Client>>,
    r_to_clients_payload_applied: Stream<
        'a,
        (u32, ReplicaPayload),
        Unbounded,
        NoTick,
        Cluster<Client>,
    >,
    flow: &FlowBuilder<'a>,
    num_clients_per_node: usize,
    median_latency_window_size: usize,
    f: usize,
) -> Stream<'a, (u32, ClientPayload), Unbounded, NoTick, Cluster<Client>> {
    let c_id = flow.cluster_self_id(clients);
    p_to_clients_leader_elected
        .clone()
        .for_each(q!(|ballot| println!(
            "Client notified that leader was elected: {:?}",
            ballot
        )));
    // r_to_clients_payload_applied.clone().inspect(q!(|payload: &(u32, ReplicaPayload)| println!("Client received payload: {:?}", payload)));
    // Only keep the latest leader
    let c_max_leader_ballot = p_to_clients_leader_elected.max();
    let c_new_leader_ballot = c_max_leader_ballot.clone().latest_tick().delta();
    // Whenever the leader changes, make all clients send a message
    let c_new_payloads_when_leader_elected =
        c_new_leader_ballot
            .clone()
            .flat_map(q!(move |leader_ballot| (0..num_clients_per_node).map(
                move |i| (
                    leader_ballot.get_id(),
                    ClientPayload {
                        key: i as u32,
                        value: c_id.to_string()
                    }
                )
            )));
    // Whenever replicas confirm that a payload was committed, collected it and wait for a quorum
    let (c_pending_quorum_payloads_complete_cycle, c_pending_quorum_payloads) =
        flow.tick_cycle(clients);
    let c_received_payloads = r_to_clients_payload_applied
        .tick_batch()
        .map(q!(|(sender, replica_payload)| (
            replica_payload.key,
            sender
        )))
        .union(c_pending_quorum_payloads);
    let c_received_quorum_payloads = c_received_payloads
        .clone()
        .fold_keyed(
            q!(|| 0),
            q!(|curr_count, _sender| {
                *curr_count += 1; // Assumes the same replica will only send commit once
            }),
        )
        .filter_map(q!(move |(key, count)| {
            if count == f + 1 {
                Some(key)
            } else {
                None
            }
        }));
    let c_new_pending_quorum_payloads =
        c_received_payloads.anti_join(c_received_quorum_payloads.clone());
    c_pending_quorum_payloads_complete_cycle.complete_next_tick(c_new_pending_quorum_payloads);
    // Whenever all replicas confirm that a payload was committed, send another payload
    let c_new_payloads_when_committed = c_received_quorum_payloads
        .clone()
        .cross_singleton(c_max_leader_ballot.clone().latest_tick())
        .map(q!(move |(key, leader_ballot)| (
            leader_ballot.get_id(),
            ClientPayload {
                key,
                value: c_id.to_string()
            }
        )));
    let c_to_proposers = c_new_payloads_when_leader_elected
        .union(c_new_payloads_when_committed)
        .all_ticks();

    // Track statistics
    let (c_timers_complete_cycle, c_timers) =
        flow.tick_cycle::<Stream<'a, (usize, SystemTime), _, _, _>>(clients);
    let c_new_timers_when_leader_elected = c_new_leader_ballot
        .map(q!(|_| SystemTime::now()))
        .flat_map(q!(
            move |now| (0..num_clients_per_node).map(move |virtual_id| (virtual_id, now))
        ));
    let c_updated_timers = c_received_quorum_payloads
        .clone()
        .map(q!(|key| (key as usize, SystemTime::now())));
    let c_new_timers = c_timers
        .clone() // Update c_timers in tick+1 so we can record differences during this tick (to track latency)
        .union(c_new_timers_when_leader_elected)
        .union(c_updated_timers.clone())
        .reduce_keyed(q!(|curr_time, new_time| {
            if new_time > *curr_time {
                *curr_time = new_time;
            }
        }));
    c_timers_complete_cycle.complete_next_tick(c_new_timers);

    let c_stats_output_timer = flow.source_interval(clients, q!(Duration::from_secs(1)));

    let c_latency_reset = c_stats_output_timer
        .clone()
        .latest_tick()
        .map(q!(|_| None))
        .defer_tick();

    let c_latencies = c_timers
        .join(c_updated_timers)
        .map(q!(|(_virtual_id, (prev_time, curr_time))| Some(
            curr_time.duration_since(prev_time).unwrap().as_micros()
        )))
        .union(c_latency_reset.into_stream())
        .all_ticks()
        .fold(
            // Create window with ring buffer using vec + wraparound index
            // TODO: Would be nice if I could use vec![] instead, but that doesn't work in HF+ with RuntimeData *median_latency_window_size
            q!(move || (
                Rc::new(RefCell::new(Vec::<u128>::with_capacity(
                    median_latency_window_size
                ))),
                0usize,
                false
            )),
            q!(move |(latencies, write_index, has_any_value), latency| {
                let mut latencies_mut = latencies.borrow_mut();
                if let Some(latency) = latency {
                    // Insert into latencies
                    if let Some(prev_latency) = latencies_mut.get_mut(*write_index) {
                        *prev_latency = latency;
                    } else {
                        latencies_mut.push(latency);
                    }
                    *has_any_value = true;
                    // Increment write index and wrap around
                    *write_index += 1;
                    if *write_index == median_latency_window_size {
                        *write_index = 0;
                    }
                } else {
                    // reset latencies
                    latencies_mut.clear();
                    *write_index = 0;
                    *has_any_value = false;
                }
            }),
        );
    let c_throughput_new_batch = c_received_quorum_payloads
        .clone()
        .count()
        .continue_unless(c_stats_output_timer.clone().latest_tick())
        .map(q!(|batch_size| (batch_size, false)));

    let c_throughput_reset = c_stats_output_timer
        .clone()
        .latest_tick()
        .map(q!(|_| (0, true)))
        .defer_tick();

    let c_throughput = c_throughput_new_batch
        .union(c_throughput_reset)
        .all_ticks()
        .fold(
            q!(|| (0, 0)),
            q!(|(total, num_ticks), (batch_size, reset)| {
                if reset {
                    *total = 0;
                    *num_ticks = 0;
                } else {
                    *total += batch_size as u32;
                    *num_ticks += 1;
                }
            }),
        );

    c_stats_output_timer
        .cross_singleton(c_latencies)
        .cross_singleton(c_throughput)
        .tick_samples()
        .for_each(q!(move |(
            (_, (latencies, _write_index, has_any_value)),
            (throughput, num_ticks),
        )| {
            let mut latencies_mut = latencies.borrow_mut();
            let median_latency = if has_any_value {
                let (_, median, _) =
                    latencies_mut.select_nth_unstable(median_latency_window_size / 2);
                *median
            } else {
                0
            };
            println!("Median latency: {}ms", median_latency as f64 / 1000.0);
            println!("Throughput: {} requests/s", throughput);
            println!("Num ticks per second: {}", num_ticks);
        }));
    // End track statistics
    c_to_proposers
}

// Proposer logic to calculate the largest ballot received so far.
fn p_max_ballot<'a>(
    flow: &FlowBuilder<'a>,
    proposers: &Cluster<Proposer>,
    a_to_proposers_p1b: Stream<'a, (u32, P1b), Unbounded, NoTick, Cluster<Proposer>>,
    a_to_proposers_p2b: Stream<'a, (u32, P2b), Unbounded, NoTick, Cluster<Proposer>>,
    p_to_proposers_i_am_leader: Stream<'a, Ballot, Unbounded, NoTick, Cluster<Proposer>>,
) -> Singleton<'a, Ballot, Unbounded, NoTick, Cluster<Proposer>> {
    let p_received_p1b_ballots = a_to_proposers_p1b
        .clone()
        .map(q!(|(_, p1b)| p1b.max_ballot));
    let p_received_p2b_ballots = a_to_proposers_p2b
        .clone()
        .map(q!(|(_, p2b)| p2b.max_ballot));
    p_received_p1b_ballots
        .union(p_received_p2b_ballots)
        .union(p_to_proposers_i_am_leader)
        .max()
        .unwrap_or(flow.singleton(proposers, q!(Ballot { num: 0, id: 0 })))
}

// Proposer logic to calculate the next ballot number. Expects p_received_max_ballot, the largest ballot received so far. Outputs streams: ballot_num, and has_largest_ballot, which only contains a value if we have the largest ballot.
#[allow(clippy::type_complexity)]
fn p_ballot_calc<'a>(
    flow: &FlowBuilder<'a>,
    proposers: &Cluster<Proposer>,
    p_received_max_ballot: Singleton<'a, Ballot, Bounded, Tick, Cluster<Proposer>>,
) -> (
    Singleton<'a, u32, Bounded, Tick, Cluster<Proposer>>,
    Optional<'a, (Ballot, u32), Bounded, Tick, Cluster<Proposer>>,
) {
    let p_id = flow.cluster_self_id(proposers);
    let (p_ballot_num_complete_cycle, p_ballot_num) =
        flow.tick_cycle_with_initial(proposers, flow.singleton(proposers, q!(0)).latest_tick());

    let p_new_ballot_num = p_received_max_ballot
        .clone()
        .cross_singleton(p_ballot_num.clone())
        .map(q!(move |(received_max_ballot, ballot_num)| {
            if received_max_ballot
                > (Ballot {
                    num: ballot_num,
                    id: p_id,
                })
            {
                received_max_ballot.num + 1
            } else {
                ballot_num
            }
        }));
    p_ballot_num_complete_cycle.complete_next_tick(p_new_ballot_num);

    let p_has_largest_ballot = p_received_max_ballot
        .clone()
        .cross_singleton(p_ballot_num.clone())
        .filter(q!(
            move |(received_max_ballot, ballot_num)| *received_max_ballot
                <= Ballot {
                    num: *ballot_num,
                    id: p_id
                }
        ));

    // End stable leader election
    (p_ballot_num, p_has_largest_ballot)
}

// Proposer logic to send "I am leader" messages periodically to other proposers, or send p1a to acceptors if other leaders expired.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn p_p1a<'a>(
    p_ballot_num: Singleton<'a, u32, Bounded, Tick, Cluster<Proposer>>,
    p_is_leader: Optional<'a, bool, Bounded, Tick, Cluster<Proposer>>,
    proposers: &Cluster<Proposer>,
    p_to_proposers_i_am_leader: Stream<'a, Ballot, Unbounded, NoTick, Cluster<Proposer>>,
    flow: &FlowBuilder<'a>,
    acceptors: &Cluster<Acceptor>,
    i_am_leader_send_timeout: u64,  // How often to heartbeat
    i_am_leader_check_timeout: u64, // How often to check if heartbeat expired
    i_am_leader_check_timeout_delay_multiplier: usize, /* Initial delay, multiplied by proposer pid, to stagger proposers checking for timeouts */
) -> (
    Stream<'a, Ballot, Unbounded, NoTick, Cluster<Proposer>>,
    Stream<'a, P1a, Unbounded, NoTick, Cluster<Acceptor>>,
) {
    let p_id = flow.cluster_self_id(proposers);
    let p_to_proposers_i_am_leader_new = p_ballot_num
        .clone()
        .continue_if(
            flow.source_interval(proposers, q!(Duration::from_secs(i_am_leader_send_timeout)))
                .latest_tick(),
        )
        .continue_if(p_is_leader.clone())
        .map(q!(move |ballot_num| Ballot {
            num: ballot_num,
            id: p_id
        }))
        .all_ticks()
        .broadcast_bincode_interleaved(proposers);

    let p_latest_received_i_am_leader = p_to_proposers_i_am_leader.clone().fold(
        q!(|| None),
        q!(|latest, _| {
            // Note: May want to check received ballot against our own?
            *latest = Some(Instant::now());
        }),
    );
    // Add random delay depending on node ID so not everyone sends p1a at the same time
    let p_leader_expired = flow.source_interval_delayed(proposers, q!(Duration::from_secs((p_id * i_am_leader_check_timeout_delay_multiplier as u32).into())), q!(Duration::from_secs(i_am_leader_check_timeout)))
        .cross_singleton(p_latest_received_i_am_leader.clone())
        .latest_tick()
        // .inspect(q!(|v| println!("Proposer checking if leader expired")))
        // .continue_if(p_is_leader.clone().count().filter(q!(|c| *c == 0)).inspect(q!(|c| println!("Proposer is_leader count: {}", c))))
        .continue_unless(p_is_leader)
        .filter(q!(move |(_, latest_received_i_am_leader)| {
            if let Some(latest_received_i_am_leader) = latest_received_i_am_leader {
                (Instant::now().duration_since(*latest_received_i_am_leader)) > Duration::from_secs(i_am_leader_check_timeout)
            } else {
                true
            }
        }));
    p_leader_expired
        .clone()
        .all_ticks()
        .for_each(q!(|_| println!("Proposer leader expired")));

    let p_to_acceptors_p1a = p_ballot_num
        .continue_if(p_leader_expired)
        .map(q!(move |ballot_num| P1a {
            ballot: Ballot {
                num: ballot_num,
                id: p_id
            }
        }))
        .all_ticks()
        .broadcast_bincode_interleaved(acceptors);
    (p_to_proposers_i_am_leader_new, p_to_acceptors_p1a)
}

#[cfg(test)]
mod tests {
    use hydroflow_plus_deploy::DeployRuntime;
    use stageleft::RuntimeData;

    #[test]
    fn paxos_ir() {
        let builder = hydroflow_plus::FlowBuilder::new();
        let _ = super::paxos(&builder, 1, 1, 1, 1, 1, 1, 1);
        let built = builder.with_default_optimize();

        insta::assert_debug_snapshot!(built.ir());

        let _ = built.compile::<DeployRuntime>(&RuntimeData::new("FAKE"));
    }
}
