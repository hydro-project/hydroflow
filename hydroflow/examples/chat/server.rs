use crate::{GraphType, Opts};

use crate::helpers::{
    connect_get_addr, deserialize_msg, is_chat_msg, is_connect_req, serialize_msg,
};
use crate::protocol::Message;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use tokio::net::UdpSocket;

pub(crate) async fn run_server(opts: Opts) {
    // First, set up the socket
    let server_socket = UdpSocket::bind(("127.0.0.1", opts.port)).await.unwrap();
    let (outbound, inbound) = hydroflow::util::udp_lines(server_socket);
    println!("Server live!");

    let mut df: Hydroflow = hydroflow_syntax! {
        // NW channels
        outbound_chan = merge() -> map(|(m,a)| (serialize_msg(m), a)) -> sink_async(outbound);
        inbound_chan = recv_stream(inbound) -> map(deserialize_msg) -> tee();
        members = inbound_chan[0] -> filter_map(is_connect_req)
                                  -> filter_map(connect_get_addr) -> tee();
        msgs = inbound_chan[1] -> filter_map(is_chat_msg);

        // Logic
        members[0] -> map(|addr| (Message::ConnectResponse, addr)) -> [0]outbound_chan;
        broadcast = cross_join() -> [1]outbound_chan;
        msgs -> [0]broadcast;
        members[1] -> [1]broadcast;
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
