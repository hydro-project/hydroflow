use std::net::SocketAddr;
use std::path::Path;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{UdpSink, UdpStream};

use crate::helpers::parse_out;
use crate::protocol::{CoordMsg, MsgType, SubordResponse};
use crate::{Addresses, GraphType};

pub(crate) async fn run_coordinator(
    outbound: UdpSink,
    inbound: UdpStream,
    path: impl AsRef<Path>,
    log: &String,
    graph: Option<GraphType>,
) {
    let mut df: Hydroflow = hydroflow_syntax! {
        // fetch subordinates from file, convert ip:port to a SocketAddr, and tee
        subords = source_json(path)
            -> flat_map(|json: Addresses| json.subordinates)
            -> map(|s| s.parse::<SocketAddr>().unwrap())
            -> tee();

        // set up channels
        msgs = source_stream_serde(inbound)
            -> map(Result::unwrap)
            -> map(|(m, _a)| m)
            -> demux(|m:SubordResponse, var_args!(commits, aborts, acks, endeds, errs)| match m.mtype {
                    MsgType::Commit => commits.give(m),
                    MsgType::Abort => aborts.give(m),
                    MsgType::AckP2 {..} => acks.give(m),
                    MsgType::Ended {..} => endeds.give(m),
                    _ => errs.give(m),
                });
        msgs[errs] -> for_each(|m| println!("Received unexpected message type: {:?}", m));
        msgs[endeds] -> null();
        log_to_disk = union() -> dest_file(log, true);

        // setup broadcast channel to all subords
        broadcast_join = cross_join() -> dest_sink_serde(outbound);
        broadcast = union() -> [0]broadcast_join;
        subords[1] -> [1]broadcast_join;
        subords[2] -> for_each(|s| println!("Subordinate: {:?}", s));

        // Phase 1 initiate:
        // Given a transaction commit request from stdio, broadcast a Prepare to subordinates
        source_stdin()
            -> filter_map(|l: Result<std::string::String, std::io::Error>| parse_out(l.unwrap()))
            -> map(|xid| CoordMsg{xid, mtype: MsgType::Prepare})
            -> [0]broadcast;

        // Phase 1 responses:
        // as soon as we get an abort message for P1, we start Phase 2 with Abort.
        // We'll respond to each abort message: this is redundant but correct (and monotone)
        msgs[aborts]
            -> map(|m: SubordResponse| CoordMsg{xid: m.xid, mtype: MsgType::Abort})
            -> [1]broadcast;

        // count commit votes
        commit_votes = msgs[commits]
            -> map(|m: SubordResponse| (m.xid, 1))
            -> fold_keyed::<'static, u16, u32>(|| 0, |acc: &mut _, val| *acc += val);

        // count subordinates
        subord_total = subords[0] -> fold::<'tick>(0, |a,_b| a+1) -> tee(); // -> for_each(|n| println!("There are {} subordinates.", n));

        // If commit_votes for this xid is the same as subord_total, send a P2 Commit message
        committed = join() -> map(|(_c, (xid, ()))| xid) -> tee();
        commit_votes -> map(|(xid, c)| (c, xid)) -> [0]committed;
        subord_total[0] -> map(|c| (c, ())) -> [1]committed;
        // Presumed abort: log commits (send commit only after flushing to disk)
        committed[0] -> map(|xid| format!("Commit {:?}", xid))
            -> [0]log_to_disk;
        committed[1] -> next_tick()
            -> map(|xid| CoordMsg{xid, mtype: MsgType::Commit})
            -> [2]broadcast;

        // count ack votes
        ack_votes = msgs[acks]
            -> map(|m: SubordResponse| (m.xid, 1))
            -> fold_keyed::<'static, u16, u32>(|| 0, |acc: &mut _, val| *acc += val);

        // If ack_votes for this xid is the same as subord_total, send a End message
        acked = join() -> map(|(_c, (xid, ()))| xid) -> tee();
        ack_votes -> map(|(xid, c)| (c, xid)) -> [0]acked;
        subord_total[1] -> map(|c| (c, ())) -> [1]acked;
        // Presumed abort: log ends (don't need to flush)
        acked[0] -> map(|xid| format!("Ended {:?}", xid))
            -> [1]log_to_disk;
        acked[1] -> map(|xid| CoordMsg{xid, mtype: MsgType::End})
            -> [3]broadcast;

        // Handler for ended acknowledgments not necessary; we just print them
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
