use std::collections::HashSet;
use std::net::SocketAddr;

use futures::join;
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{bind_udp_bytes};

use crate::helpers::{get_phase_1_addr, get_phase_2_addr};
use crate::protocol::*;
use crate::{GraphType};

pub(crate) async fn run_acceptor(
    addr: SocketAddr,
    graph: Option<GraphType>,
) {
    let phase_1_addr = get_phase_1_addr(addr);
    let phase_2_addr = get_phase_2_addr(addr);

    let p1a_future = bind_udp_bytes(phase_1_addr);
    let p2a_future = bind_udp_bytes(phase_2_addr);
    let ((p1b_sink, p1a_src, _),
        (p2b_sink, p2a_src, _))
        = join!(p1a_future, p2a_future);

    let mut df: Hydroflow = hydroflow_syntax! {
        // define inputs/outputs
        p1a = source_stream_serde(p1a_src)
            -> map(Result::unwrap)
            -> inspect(|(m, a)| println!("Received {:?} from {:?}", m, a))
            -> tee();
        p1b = cross_join::<'tick>()
            -> map(|(((p1a, proposer), log), max_ballot): (((P1a, SocketAddr), HashSet<Entry>), Ballot)| 
            (P1b {
                ballot: p1a.ballot,
                max_ballot,
                log,
            }, proposer))
            -> inspect(|(m, a)| println!("Sending {:?} to {:?}", m, a))
            -> dest_sink_serde(p1b_sink);
        p2a = source_stream_serde(p2a_src)
            -> map(Result::unwrap)
            -> inspect(|(m, a)| println!("Received {:?} from {:?}", m, a))
            -> tee();
        p2b = cross_join::<'tick>()
            -> map(|((p2a, proposer), max_ballot):((P2a, SocketAddr), Ballot)|
            (P2b {
                entry: p2a.entry,
                max_ballot,
            }, proposer))
            -> inspect(|(m, a)| println!("Sending {:?} to {:?}", m, a))
            -> dest_sink_serde(p2b_sink);

        // find max_ballot
        ballots = persist();
        max_ballot = ballots -> reduce(|mut curr_max:Ballot, ballot:Ballot| {
            if ballot > curr_max {
                curr_max = ballot;
            }
            curr_max
        }) -> tee();

        p1a[0] -> map(|(m, _): (P1a, SocketAddr)| m.ballot) -> ballots;

        // find max entry for each slot in the log
        log = persist()
            -> map(|e: Entry| (e.slot, e))
            -> reduce_keyed::<'tick>(|curr_max:&mut Entry, e:Entry| {
                if e.ballot > curr_max.ballot {
                    *curr_max = e;
                }
            })
            -> fold::<'tick>(HashSet::new(), |mut log:HashSet<Entry>, (_slot, e): (u16, Entry)| {
                log.insert(e);
                log
            });
        
        // reply with p1b
        p1a_and_log = cross_join::<'tick>() -> [0]p1b;
        p1a[1] -> [0]p1a_and_log;
        log -> [1]p1a_and_log;
        max_ballot[0] -> [1]p1b;

        // write p2a messages to log
        p2a_and_max_ballot = cross_join::<'tick>();
        p2a[0] -> [0]p2a_and_max_ballot;
        max_ballot[1] -> [1]p2a_and_max_ballot;
        p2a_and_max_ballot -> filter_map(|((m, _proposer), max_ballot)| {
            if m.entry.ballot >= max_ballot {
                Some(m.entry)
            } else {
                None
            }
        }) -> log;

        // reply with p2b
        p2a[1] -> [0]p2b;
        max_ballot[2] -> [1]p2b;
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
