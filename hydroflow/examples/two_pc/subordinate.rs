use crate::helpers::{
    decide, deserialize_msg, is_coord_p2, is_end_msg, is_prepare_msg, serialize_msg,
};
use crate::protocol::{CoordMsg, MsgType, SubordResponse};
use crate::{GraphType, Opts};
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub(crate) async fn run_subordinate(opts: Opts, coordinator: String) {
    // setup message send/recv ports
    let socket = UdpSocket::bind(("127.0.0.1", opts.port)).await.unwrap();
    let my_addr = socket.local_addr().unwrap();
    let (outbound, inbound) = hydroflow::util::udp_lines(socket);
    println!("Coordinator: {}", coordinator);
    let server_addr = coordinator.trim().parse::<SocketAddr>().unwrap();

    let mut df: Hydroflow = hydroflow_syntax! {
         // set up channels
        outbound_chan = merge() -> tee();
        outbound_chan[0] -> map(|(m,a)| (serialize_msg(m), a)) -> sink_async(outbound);
        inbound_chan = recv_stream(inbound) -> map(deserialize_msg) -> tee();
        prepare_chan = inbound_chan[0] -> filter_map(is_prepare_msg);
        p2_chan = inbound_chan[1] -> filter_map(is_coord_p2);
        end_chan = inbound_chan[2] -> filter_map(is_end_msg);

        // we log all messages (in this prototype we just print)
        inbound_chan[3] -> for_each(|m| println!("Received {:?}", m));
        outbound_chan[1] -> for_each(|m| println!("Sending {:?}", m));


        // handle p1 message: choose vote and respond
        // in this prototype we choose randomly whether to abort via decide()
        report_chan = prepare_chan -> map(|m: CoordMsg| (
            SubordResponse{
                xid: m.xid,
                addr: my_addr.to_string(),
                mtype: if decide(67) {MsgType::Commit} else {MsgType::Abort}
            },
            server_addr));
        report_chan -> [0]outbound_chan;

        // handle p2 message: acknowledge (and print)
        p2_response = map(|(m, t)| (SubordResponse{
            xid: m.xid,
            addr: my_addr.to_string(),
            mtype: t,
        }, server_addr)) -> [1]outbound_chan;
        p2_chan -> map(|m:CoordMsg| (m, MsgType::AckP2)) -> p2_response;

        // handle end message: acknowledge (and print)
        end_chan -> map(|m:CoordMsg| (SubordResponse{
            xid: m.xid,
            addr: my_addr.to_string(),
            mtype: MsgType::Ended,
        }, server_addr)) -> [2]outbound_chan;


    };

    if let Some(graph) = opts.graph {
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
