use crate::{GraphType, Opts};
use hydroflow::pusherator::Pusherator;

use crate::helpers::{
    connect_get_addr, deserialize_msg, resolve_ipv4_connection_addr, serialize_msg,
};
use crate::protocol::Message;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use tokio::net::UdpSocket;

pub(crate) async fn run_server(opts: Opts) {
    // First, set up the socket

    let server_addr = resolve_ipv4_connection_addr(opts.addr, opts.port)
        .expect("Unable to bind to provided IP and port");
    let server_socket = UdpSocket::bind(server_addr).await.unwrap();
    let (outbound, inbound) = hydroflow::util::udp_lines(server_socket);
    println!("Listening on {}", server_addr);
    println!("Server live!");

    let mut df: Hydroflow = hydroflow_syntax! {
        // NW channels
        outbound_chan = merge() -> map(|(m,a)| (serialize_msg(m), a)) -> sink_async(outbound);
        inbound_chan = recv_stream(inbound) -> map(deserialize_msg)
            ->  demux(|m, tl!(members, msgs, errs)|
                    match m {
                        Message::ConnectRequest {..} => members.give(m),
                        Message::ChatMsg {..} => msgs.give(m),
                        _ => errs.give(m),
                    }
                );
        members = inbound_chan[members] -> filter_map(connect_get_addr) -> tee();
        inbound_chan[errs] -> for_each(|m| println!("Received unexpected message type: {:?}", m));

        // Logic
        members[0] -> map(|addr| (Message::ConnectResponse, addr)) -> [0]outbound_chan;
        broadcast = cross_join() -> [1]outbound_chan;
        inbound_chan[msgs] -> [0]broadcast;
        members[1] -> [1]broadcast;
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
