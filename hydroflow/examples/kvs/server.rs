use crate::{GraphType, Opts};

use crate::helpers::{
    deserialize_msg, is_get, is_put, resolve_ipv4_connection_addr, serialize_msg,
};
use crate::protocol::KVSMessage;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use std::net::SocketAddr;
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
        inbound_chan = recv_stream(inbound) -> map(deserialize_msg) -> tee();
        puts = inbound_chan[0] -> filter_map(is_put) -> tee();
        gets = inbound_chan[1] -> filter_map(is_get) -> tee();

        puts[0] -> for_each(|m| println!("Got a Put: {:?}", m));
        gets[0] -> for_each(|m| println!("Got a Get: {:?}", m));

        parsed_puts = puts[1] -> filter_map(|m| {
            match m {
                KVSMessage::Put{client, key, value} => Some((client, key, value)),
                _ => None }
            }) -> tee();
        parsed_gets = gets[1] -> filter_map(|m| {
            match m {
                KVSMessage::Get{client, key} => Some((key, client)),
                _ => None }
            });

        // ack puts
        parsed_puts[0] -> map(|(client, key, value): (SocketAddr, String, String)| (KVSMessage::Response{key, value}, client))
            -> [0]outbound_chan;

        // join PUTs and GETs  by (key, client)
        lookup = join()->tee();
        parsed_puts[1] -> map(|pp| (pp.1, pp.2)) -> [0]lookup;
        parsed_gets -> [1]lookup;
        lookup[0] -> for_each(|t| println!("Found a match: {:?}", t));

        // send lookup responses back to the client address from the GET
        lookup[1] -> map(|(key, (value, client))| (KVSMessage::Response{key, value}, client)) -> [1]outbound_chan;
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
