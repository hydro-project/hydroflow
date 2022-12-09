use std::net::SocketAddr;

use crate::helpers::decide;
use crate::protocol::{CoordMsg, MsgType, SubordResponse};
use crate::GraphType;
use hydroflow::hydroflow_syntax;
use hydroflow::pusherator::Pusherator;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{UdpSink, UdpStream};

pub(crate) async fn run_subordinate(
    outbound: UdpSink,
    inbound: UdpStream,
    server_addr: SocketAddr,
    graph: Option<GraphType>,
) {
    let mut df: Hydroflow = hydroflow_syntax! {
         // set up channels
        outbound_chan = merge() -> tee();
        outbound_chan[0] -> sink_async_serde(outbound);
        inbound_chan = source_stream_serde(inbound) -> map(|(m, _a)| m) -> tee();
        msgs = inbound_chan[0] ->  demux(|m:CoordMsg, tl!(prepares, p2, ends, errs)| match m.mtype {
            MsgType::Prepare => prepares.give(m),
            MsgType::Abort => p2.give(m),
            MsgType::Commit => p2.give(m),
            MsgType::End {..} => ends.give(m),
            _ => errs.give(m),
        });
        msgs[errs] -> for_each(|m| println!("Received unexpected message type: {:?}", m));

        // we log all messages (in this prototype we just print)
        inbound_chan[1] -> for_each(|m| println!("Received {:?}", m));
        outbound_chan[1] -> for_each(|m| println!("Sending {:?}", m));


        // handle p1 message: choose vote and respond
        // in this prototype we choose randomly whether to abort via decide()
        report_chan = msgs[prepares] -> map(|m: CoordMsg| (
            SubordResponse{
                xid: m.xid,
                mtype: if decide(67) {MsgType::Commit} else {MsgType::Abort}
            },
            server_addr));
        report_chan -> [0]outbound_chan;

        // handle p2 message: acknowledge (and print)
        p2_response = map(|(m, t)| (SubordResponse{
            xid: m.xid,
            mtype: t,
        }, server_addr)) -> [1]outbound_chan;
        msgs[p2] -> map(|m:CoordMsg| (m, MsgType::AckP2)) -> p2_response;

        // handle end message: acknowledge (and print)
        msgs[ends] -> map(|m:CoordMsg| (SubordResponse{
            xid: m.xid,
            mtype: MsgType::Ended,
        }, server_addr)) -> [2]outbound_chan;


    };

    if let Some(graph) = graph {
        let serde_graph = df
            .serde_graph()
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
