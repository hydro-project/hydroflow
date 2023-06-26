use std::net::SocketAddr;
use std::path::Path;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{UdpSink, UdpStream};

use crate::protocol::{CoordMsg, MsgType, SubordResponse};
use crate::{Addresses, GraphType};

pub(crate) async fn run_subordinate(
    outbound: UdpSink,
    inbound: UdpStream,
    path: impl AsRef<Path>,
    graph: Option<GraphType>,
) {
    let mut df: Hydroflow = hydroflow_syntax! {
        // Outbound address
        server_addr = source_json(path)
            -> map(|json: Addresses| json.coordinator)
            -> map(|s| s.parse::<SocketAddr>().unwrap())
            -> inspect(|coordinator| println!("Coordinator: {}", coordinator));

        // set up channels
        outbound_chan = cross_join() -> dest_sink_serde(outbound);
        server_addr -> [1]outbound_chan;
        inbound_chan = source_stream_serde(inbound) -> map(Result::unwrap) -> map(|(m, _a)| m);
        msgs = inbound_chan ->  demux(|m:CoordMsg, var_args!(vote_reqs, errs)| match m.mtype {
            MsgType::VoteReq => vote_reqs.give(m),
            _ => errs.give(m),
        });
        msgs[errs] -> for_each(|m| println!("Received unexpected message type: {:?}", m));

        // handle p1 message: choose vote and respond
        // in this prototype we choose randomly whether to abort via decide()
        report_chan = msgs[vote_reqs] -> map(|m: CoordMsg| SubordResponse {
            xid: m.xid,
            mtype: MsgType::Vote,
        });
        report_chan -> [0]outbound_chan;
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
