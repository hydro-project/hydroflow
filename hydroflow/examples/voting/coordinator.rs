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
    graph: Option<GraphType>,
) {
    let mut df: Hydroflow = hydroflow_syntax! {
        // fetch subordinates from file, convert ip:port to a SocketAddr, and tee
        subords = source_json(path)
            -> flat_map(|json: Addresses| json.subordinates)
            -> map(|s| s.parse::<SocketAddr>().unwrap())
            -> tee();

        // set up channels
        outbound_chan = dest_sink_serde(outbound);
        inbound_chan = source_stream_serde(inbound) -> map(Result::unwrap) -> map(|(m, _a)| m);
        msgs = inbound_chan ->  demux(|m:SubordResponse, var_args!(votes, errs)| match m.mtype {
                    MsgType::Vote => votes.give(m),
                    _ => errs.give(m),
                });
        msgs[errs] -> for_each(|m| println!("Received unexpected message type: {:?}", m));

        // setup broadcast channel to all subords
        broadcast = cross_join() -> outbound_chan;
        subords[1] -> [1]broadcast;
        subords[2] -> for_each(|s| println!("Subordinate: {:?}", s));

        // Phase 1 initiate:
        // Given a transaction commit request from stdio, broadcast a Prepare to subordinates
        source_stdin()
            -> filter_map(|l: Result<std::string::String, std::io::Error>| parse_out(l.unwrap()))
            -> map(|xid| CoordMsg{xid, mtype: MsgType::VoteReq})
            -> [0]broadcast;

        // count votes
        votes = msgs[votes]
            -> map(|m: SubordResponse| (m.xid, 1))
            -> fold_keyed::<'static, u16, u32>(|| 0, |acc: &mut _, val| *acc += val);

        // count subordinates
        subord_total = subords[0] -> fold::<'tick>(0, |a,_b| a+1); // -> for_each(|n| println!("There are {} subordinates.", n));

        // If commit_votes for this xid is the same as all_votes, output committed
        committed = join() -> map(|(_c, (xid, ()))| xid) -> for_each(|xid| println!("Committed: {:?}", xid));
        votes -> map(|(xid, c)| (c, xid)) -> [0]committed;
        subord_total -> map(|c| (c, ())) -> [1]committed;
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
