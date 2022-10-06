use crate::protocol::{deserialize_msg, serialize_msg, CoordMsg, MsgType, SubordResponse};
use crate::{GraphType, Opts};
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::io::AsyncBufReadExt;
use tokio::net::UdpSocket;
use tokio_stream::wrappers::LinesStream;

pub(crate) async fn run_subordinate(opts: Opts, coordinator: String) {
    // setup message send/recv ports
    let socket = UdpSocket::bind(("127.0.0.1", opts.port)).await.unwrap();
    let my_addr = socket.local_addr().unwrap();
    let (outbound, inbound) = hydroflow::util::udp_lines(socket);
    println!("Coordinator: {}", coordinator);
    let server_addr = coordinator.trim().parse::<SocketAddr>().unwrap();

    let mut df: Hydroflow = hydroflow_syntax! {
         // set up channels
        outbound_chan = merge() -> sink_async(outbound);
        inbound_chan = recv_stream(inbound) -> map(|m| deserialize_msg(m)) -> tee();
        prepare_chan = inbound_chan[0] -> filter_map(|m: CoordMsg| match m.mtype {
                    MsgType::Prepare => Some(m),
                    _ => None }) -> tee();
        // p2_commit_chan = inbound_chan[1] -> filter_map(|m: CoordMsg| match m.mtype {
        //         MsgType::Commit => Some(m),
        //         _ => None });
        // p2_abort_chan = inbound_chan[2] -> filter_map(|m: CoordMsg| match m.mtype {
        //         MsgType::Abort => Some(m),
        //         _ => None });
        // end_chan = inbound_chan[3] -> filter_map(|m: CoordMsg| match m.mtype {
        //         MsgType::End => Some(m),
        //         _ => None });

        // receive p1 message
        prepare_chan[0] -> map(|m: CoordMsg| (serialize_msg(SubordResponse{xid: m.xid,
            mid: m.mid,
            addr: my_addr.to_string(),
            mtype: MsgType::Commit,}), server_addr))
            -> [0]outbound_chan;
        prepare_chan[1] -> for_each(|m| println!("received prepare message {:?}", m));





    };

    if let Some(graph) = opts.graph {
        match graph {
            GraphType::Mermaid => {
                println!("{}", df.generate_mermaid())
            }
            GraphType::Dot => {
                println!("{}", df.generate_dot())
            }
            GraphType::Json => {
                println!("{}", df.generate_json())
            }
        }
    }

    df.run_async().await.unwrap();
}
