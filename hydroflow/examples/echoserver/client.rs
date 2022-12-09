use crate::protocol::EchoMsg;
use crate::GraphType;
use chrono::prelude::*;
use hydroflow::hydroflow_syntax;
use hydroflow::util::{UdpSink, UdpStream};
use std::net::SocketAddr;

pub(crate) async fn run_client(
    outbound: UdpSink,
    inbound: UdpStream,
    server_addr: SocketAddr,
    graph: Option<GraphType>,
) {
    println!("Attempting to connect to server at {:?}", server_addr);
    println!("Client live!");

    let mut flow = hydroflow_syntax! {
        // take stdin and send to server as an Echo::Message
        lines = source_stdin() -> map(|l| (EchoMsg{ payload: l.unwrap(), ts: Utc::now(), }, server_addr) )
            -> dest_sink_serde(outbound);

        // receive and print messages
        source_stream_serde(inbound) -> for_each(|(m, _a): (EchoMsg, SocketAddr) | println!("{:?}", m));
    };

    if let Some(graph) = graph {
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
