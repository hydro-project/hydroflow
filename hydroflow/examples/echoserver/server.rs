use crate::{GraphType, Opts};
use chrono::prelude::*;

use crate::helpers::{deserialize_msg, resolve_ipv4_connection_addr, serialize_msg};
use crate::protocol::{EchoMsg, EchoResponse};

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use tokio::net::UdpSocket;

pub(crate) async fn run_server(opts: Opts) {
    // First, set up the server socket
    let server_addr = resolve_ipv4_connection_addr(opts.addr, opts.port)
        .expect("Unable to bind to provided IP and port");
    let server_socket = UdpSocket::bind(server_addr).await.unwrap();
    let (outbound, inbound) = hydroflow::util::udp_lines(server_socket);
    println!("Listening on {}", server_addr);

    println!("{:?} live!", opts.role);

    let mut flow: Hydroflow = hydroflow_syntax! {
        // NW channels
        outbound_chan = map(|(m,a)| (serialize_msg(m), a)) -> sink_async(outbound);
        inbound_chan = recv_stream(inbound) -> map(deserialize_msg) -> tee();

        // Logic
        inbound_chan[0] -> for_each(|m| println!("Got {:?}", m));
        inbound_chan[1] -> map(|EchoMsg{nonce, payload, addr}| (EchoResponse{nonce, payload, ts: Utc::now()}, addr))
            -> [0]outbound_chan;
    };

    if let Some(graph) = opts.graph {
        let serde_graph = flow
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

    flow.run_async().await.unwrap();
}
