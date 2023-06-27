use std::cmp;
use std::net::SocketAddr;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use futures::join;
use hydroflow::compiled::pull::HalfMultisetJoinState;
use hydroflow::hydroflow_syntax;
use hydroflow::lattices::{DomPair, Max};
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::Persistence::*;
use hydroflow::util::{bind_udp_bytes, UdpSink, UdpStream};

use crate::helpers::{get_config, get_phase_1_addr, get_phase_2_addr, periodic};
use crate::protocol::*;
use crate::{Config, GraphType};

pub(crate) async fn run_proposer(
    addr: SocketAddr,
    path: impl AsRef<Path>,
    graph: Option<GraphType>,
) {
    // init default values
    let start_ballot_num: u16 = 0;
    let start_slot: u16 = 0;
    let noop = "";

    // create channels
    let stable_leader_addr = addr;
    let phase_1_addr = get_phase_1_addr(addr);
    let phase_2_addr = get_phase_2_addr(addr);
    let leader_future = bind_udp_bytes(stable_leader_addr);
    let p1a_future = bind_udp_bytes(phase_1_addr);
    let p2a_future = bind_udp_bytes(phase_2_addr);
    let (
        (stable_leader_sink, stable_leader_src, _),
        (p1a_sink, p1b_src, _),
        (p2a_sink, p2b_src, _),
    ) = join!(leader_future, p1a_future, p2a_future);

    let id = addr.port(); // TODO assumes that each proposer has a different port. True locally, but may not be true in distributed setting
    let config = get_config(path);
    let f = config.f;

    // create timeout triggers
    let is_node_0 = addr == config.proposers[0].parse::<SocketAddr>().unwrap();
    println!("is_node_0: {:?}", is_node_0);
    let i_am_leader_resend_trigger = periodic(config.i_am_leader_resend_timeout);
    let i_am_leader_check_timeout = if is_node_0 {
        config.i_am_leader_check_timeout_node_0
    } else {
        config.i_am_leader_check_timeout_other_nodes
    };
    let i_am_leader_check_trigger = periodic(i_am_leader_check_timeout);

    let mut df: Hydroflow = hydroflow_syntax! {
        // define inputs/outputs
        // stable leader election messages
        proposers = source_iter(config.proposers)
            // filter out self
            -> filter_map(|s| {
                let proposer = s.parse::<SocketAddr>().unwrap();
                if proposer != addr {
                    Some(proposer)
                } else {
                    None
                }
            })
            -> inspect(|p| println!("Proposer: {:?}", p))
            -> persist();
        leader_recv = source_stream_serde(stable_leader_src)
            -> map(Result::unwrap)
            -> inspect(|(m, a)| println!("Received {:?} from {:?}", m, a))
            -> map(|(m, _)| m)
            -> tee();
        leader_send = cross_join::<'tick>()
            -> inspect(|(m, a)| println!("Sending {:?} to {:?}", m, a))
            -> dest_sink_serde(stable_leader_sink);
        proposers -> [1]leader_send;
        // messages to/from acceptors
        acceptors = source_iter(config.acceptors)
            -> map(|s| s.parse::<SocketAddr>().unwrap())
            -> inspect(|a| println!("Acceptor: {:?}", a))
            -> persist()
            -> tee();
        p1a = cross_join::<'tick>()
            -> inspect(|(m, a)| println!("Sending {:?} to {:?}", m, a))
            -> dest_sink_serde(p1a_sink);
        acceptors[0] -> map(|s| get_phase_1_addr(s)) -> [1]p1a;
        p1b = source_stream_serde(p1b_src)
            -> map(Result::unwrap)
            -> inspect(|(m, a)| println!("Received {:?} from {:?}", m, a))
            -> map(|(m, _)| m)
            -> persist()
            -> tee();
        p2a_out = cross_join::<'tick>()
            -> inspect(|(m, a)| println!("Sending {:?} to {:?}", m, a))
            -> dest_sink_serde(p2a_sink);
        p2a = union() -> [0]p2a_out;
        acceptors[1] -> map(|s| get_phase_2_addr(s)) -> [1]p2a_out;
        p2b_in = source_stream_serde(p2b_src)
            -> map(Result::unwrap)
            -> inspect(|(m, a)| println!("Received {:?} from {:?}", m, a))
            -> map(|(m, _)| Persist(m))
            -> [0]p2b;
        p2b = union()
            -> persist_mut()
            -> tee();

        client_in = source_stdin()
            -> filter_map(|m: Result<String, std::io::Error>| m.ok())
            -> inspect(|m| println!("Client input: {:?}", m));

        // compute ballot (default ballot num = 0)
        ballot_num = union()
            -> map(Max::new)
            -> lattice_merge::<'static, Max<u16>>()
            -> map(|lattice: Max<u16>| lattice.0);
        source_iter(vec![start_ballot_num]) -> [0]ballot_num;
        ballot = ballot_num -> map(|num| Ballot {id, num}) -> tee();


        /////////////////////////////////////////////////////////////////////// stable leader election
        // check if we have the largest ballot
        leader_recv[0] -> map(|m: IAmLeader| m.ballot) -> received_ballots[0];
        p1b[0] -> map(|m: P1b| m.max_ballot) -> received_ballots[1];
        p2b[0] -> map(|m: P2b| m.max_ballot) -> received_ballots[2];
        received_ballots = union()
            -> map(Max::new)
            -> lattice_merge::<'static, Max<Ballot>>()
            -> map(|lattice: Max<Ballot>| lattice.0)
            -> tee();
        received_ballots[0] -> [0]has_largest_ballot;
        ballot[0] -> [1]has_largest_ballot;
        has_largest_ballot = cross_join::<'tick>() -> filter_map(|(max_received_ballot, ballot): (Ballot, Ballot)| {
            if ballot >= max_received_ballot {
                Some(())
            }
            else {
                None
            }
        }) -> tee();

        // send heartbeat if we're the leader
        source_stream(i_am_leader_resend_trigger) -> [0]leader_and_resend_timeout;
        is_leader[0] -> [1]leader_and_resend_timeout;
        leader_and_resend_timeout = cross_join::<'tick>() -> map(|_| ());
        leader_and_resend_timeout -> [0]leader_and_resend_timeout_ballot;
        ballot[1] -> [1]leader_and_resend_timeout_ballot;
        leader_and_resend_timeout_ballot = cross_join::<'tick>() -> map(|(_, b): ((), Ballot)| IAmLeader {ballot: b}) -> [0]leader_send;

        // track latest heartbeat
        source_iter(vec![0]) -> [0]latest_heartbeat;
        latest_heartbeat = union()
            -> map(Max::new)
            -> lattice_merge::<'static, Max<u64>>()
            -> map(|lattice: Max<u64>| lattice.0);
        leader_recv[1] -> map(|_| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()) -> [1]latest_heartbeat;

        // if there was no previous heartbeat when the i_am_leader_check triggers again, send p1a
        i_am_leader_check = source_stream(i_am_leader_check_trigger)
            -> map(|_| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs())
            -> inspect(|_| println!("I am leader check triggered"))
            -> [0]p1a_trigger_and_no_heartbeat;
        latest_heartbeat -> [1]p1a_trigger_and_no_heartbeat;
        p1a_trigger_and_no_heartbeat = cross_join::<'tick>()
            -> filter_map(|(check_time, heartbeat): (u64, u64)| {
                if heartbeat + u64::from(i_am_leader_check_timeout) < check_time {
                    Some(())
                }
                else {
                    None
                }
            })
            -> inspect(|_| println!("p1a trigger and no heartbeat"));
        p1a_trigger_and_no_heartbeat -> [pos]leader_expired;
        is_leader[1] -> [neg]leader_expired;
        leader_expired = difference::<'tick, 'tick>() -> inspect(|_| println!("leader expired"));
        leader_expired -> [0]leader_expired_ballot;
        ballot[2] -> [1]leader_expired_ballot;
        leader_expired_ballot = cross_join::<'tick>() -> inspect(|_| println!("leader expired ballot")) -> map(|(_, b):((), Ballot)| P1a {ballot: b}) -> [0]p1a;

        // increment ballot num if we don't have the largest ballot
        received_ballots[1] -> map(|b: Ballot| ((), b)) -> [pos]next_ballot;
        has_largest_ballot[0] -> [neg]next_ballot;
        next_ballot = anti_join() -> map(|(_, b): ((), Ballot)| b.num + 1) -> next_tick() -> [1]ballot_num;


        /////////////////////////////////////////////////////////////////////// reconcile p1b log with local log
        // check if we've received a quorum of p1bs
        p1b[1] -> map(|m: P1b| (m.ballot, m)) -> [0]relevant_p1bs;
        ballot[3] -> map(|b: Ballot| (b, ())) -> [1]relevant_p1bs;
        relevant_p1bs = join::<'tick, HalfMultisetJoinState>() -> map(|(b, (m, _)): (Ballot, (P1b, ()))| m) -> tee();
        num_p1bs = relevant_p1bs[0] -> fold::<'tick>(0, |mut sum: u16, m: P1b| sum + 1);
        p1b_quorum_reached = num_p1bs  -> filter_map(|num_p1bs: u16| if num_p1bs >= config.f + 1 {Some(())} else {None});

        p1b_quorum_reached -> [0]is_leader;
        has_largest_ballot[1] -> [1]is_leader;
        is_leader = cross_join::<'tick>() -> map(|_| ()) -> tee();

        // re-propose what was not committed
        p1b_log = relevant_p1bs[1] -> flat_map(|m: P1b| m.log) -> tee();
        p1b_uncommitted = p1b_log[0] -> map(|e: Entry| (e, 1))
            -> reduce_keyed::<'tick>(|sum: &mut u16, _| *sum += 1)
            -> filter_map(|(e, num): (Entry, u16)| {
                if num <= f + 1 {
                    Some((e.slot, e))
                }
                else {
                    None
                }
            });
        p1b_largest_uncommitted = p1b_uncommitted -> reduce_keyed::<'tick>(|largest_e: &mut Entry, e: Entry| {
            if e.ballot > largest_e.ballot {
                *largest_e = e;
            }
        });
        is_leader[2] -> map(|()| true) -> [pos]leader_and_reproposing;
        next_slot[0] -> fold::<'tick>(false, |_, _| true) -> [neg]leader_and_reproposing;
        leader_and_reproposing = difference::<'tick, 'tick>() -> map(|b| ()) -> tee(); // type: ()
        leader_and_reproposing[0] -> [0]leader_and_reproposing_and_ballot;
        ballot[4] -> [1]leader_and_reproposing_and_ballot;
        leader_and_reproposing_and_ballot = cross_join::<'tick>() // type: ((), ballot)
            -> map(|((), b): ((), Ballot)| b)
            -> tee();
        leader_and_reproposing_and_ballot[0] -> [0]leader_and_reproposing_and_ballot_and_largest_uncommitted;
        p1b_largest_uncommitted -> map(|(slot, e): (u16, Entry)| e) -> [1]leader_and_reproposing_and_ballot_and_largest_uncommitted;
        leader_and_reproposing_and_ballot_and_largest_uncommitted = cross_join::<'tick>() // type: (ballot, entry)
            -> map(|(b, e): (Ballot, Entry)| P2a {
                entry: Entry {
                    payload: e.payload,
                    slot: e.slot,
                    ballot: b,
                },
            }) -> [0]p2a;

        // hole-filling
        p1b_log[2] -> map(|m: Entry| m.slot) -> [0]proposed_slots;
        source_iter(vec![start_slot]) -> persist() -> [1]proposed_slots;
        proposed_slots = union() -> tee();
        max_proposed_slot = proposed_slots[0] -> reduce::<'tick>(|mut max: u16, s: u16| cmp::max(max, s)) -> tee();
        prev_slots = max_proposed_slot[0] -> flat_map(|max: u16| 0..max);
        prev_slots -> [pos]holes;
        proposed_slots[1] -> [neg]holes;
        leader_and_reproposing_and_ballot[1] -> [0]leader_and_reproposing_and_ballot_and_holes;
        holes = difference() -> [1]leader_and_reproposing_and_ballot_and_holes;
        leader_and_reproposing_and_ballot_and_holes = cross_join::<'tick>()
            -> map(|(b, hole): (Ballot, u16)| P2a {
                entry: Entry {
                    payload: noop.to_string(),
                    slot: hole,
                    ballot: b,
                },
            }) -> [1]p2a;

        leader_and_reproposing[1] -> [0]leader_and_reproposing_and_max_slot;
        max_proposed_slot[1] -> [1]leader_and_reproposing_and_max_slot;
        leader_and_reproposing_and_max_slot = cross_join::<'tick>()
            -> map(|((), s): ((), u16)| s + 1)
            -> next_tick()
            -> [0]new_slots;
        ballot[5] -> [0]ballot_and_next_slot;
        new_slots = union() -> [1]ballot_and_next_slot;
        // store the next slot for each ballot. Can discard slots for older ballots, so use DomPair lattice
        ballot_and_next_slot = cross_join::<'tick>()
            -> map(|(ballot, slot): (Ballot, u16)| DomPair::new(Max::new(ballot), Max::new(slot)))
            -> lattice_merge::<'static, DomPair<Max<Ballot>, Max<u16>>>()
            -> map(|lattice: DomPair<Max<Ballot>, Max<u16>>| (lattice.key.0, lattice.val.0))
            -> [0]next_slot;
        ballot[6] -> map(|b: Ballot| (b, ())) -> [1]next_slot;
        // find the next slot for the current ballot
        next_slot = join::<'tick>()
            -> map(|(ballot, (slot, ())): (Ballot, (u16, ()))| slot)
            -> tee();


        /////////////////////////////////////////////////////////////////////// send p2as
        // assign a slot
        indexed_payloads = client_in -> enumerate::<'tick>();
        is_leader[3] -> [0]leader_and_slot;
        next_slot[1] -> [1]leader_and_slot;
        leader_and_slot = cross_join::<'tick>() -> map(|((), s): ((), u16)| s);
        leader_and_slot -> [0]leader_and_slot_and_ballot;
        ballot[7] -> [1]leader_and_slot_and_ballot;
        leader_and_slot_and_ballot = cross_join::<'tick>(); // type: (slot, ballot)
        leader_and_slot_and_ballot -> [0]leader_and_slot_and_ballot_and_payload;
        indexed_payloads -> [1]leader_and_slot_and_ballot_and_payload;
        leader_and_slot_and_ballot_and_payload = cross_join::<'tick>() // type: ((slot, ballot), (index, payload))
            -> map(|((slot, ballot), (index, payload)): ((u16, Ballot), (u16, String))| P2a {
                entry: Entry {
                    payload,
                    slot: slot + index,
                    ballot,
                },
            }) -> tee();
        leader_and_slot_and_ballot_and_payload[0] -> [2]p2a;

        // increment the slot if a payload was chosen
        num_payloads = leader_and_slot_and_ballot_and_payload[1]
            -> fold::<'tick>(0, |mut sum: u16, _| sum + 1)
            -> filter(|s: &u16| *s > 0); // avoid emitting 0, even if it's correct, since it will trigger the calculation of a new slot each tick
        next_slot[2] -> [0]slot_plus_num_indexed;
        num_payloads -> [1]slot_plus_num_indexed;
        slot_plus_num_indexed = cross_join::<'tick>()
            -> map(|(slot, num): (u16, u16)| slot + num)
            -> next_tick()
            -> [1]new_slots;

        /////////////////////////////////////////////////////////////////////// process p2bs
        all_commit = p2b[1] -> map(|m: P2b| (m, 1))
            -> reduce_keyed::<'tick>(|sum: &mut u16, _| *sum += 1)
            -> filter_map(|(m, num): (P2b, u16)| {
                if num == 2*f+1 { // only count when all acceptors have accepted, so we avoid duplicate outputs (when f+1, then f+2, etc accept)
                    Some(m)
                }
                else {
                    None
                }
            })
            -> inspect(|m: &P2b| println!("Committed slot {:?}: {:?}", m.entry.slot, m.entry.payload))
            -> map(|m: P2b| Delete(m)) // delete p2bs for committed slots
            -> next_tick()
            -> [1]p2b;
    };

    if let Some(graph) = graph {
        let serde_graph = df
            .meta_graph()
            .expect("No graph found, maybe failed to parse.");
        match graph {
            GraphType::Mermaid => {
                println!("{}", serde_graph.to_mermaid());
            }
            GraphType::Dot => {
                println!("{}", serde_graph.to_dot())
            }
            GraphType::Json => {
                unimplemented!();
                // println!("{}", serde_graph.to_json())
            }
        }
    }

    df.run_async().await.unwrap();
}
