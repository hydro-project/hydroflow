use crate::{GraphType, Opts};
use chrono::prelude::*;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

use crate::protocol::{EchoMsg, EchoResponse};
use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{deserialize_msg, serialize_msg};

pub(crate) async fn run_server(opts: Opts, server_addr: SocketAddr) {
    println!("Listening on {}", server_addr);

    println!("{:?} live!", opts.role);

    let mut flow: Hydroflow = hydroflow_syntax! {
        // NW channels
        outbound_chan = map(|(m,a)| (serialize_msg(m), a)) -> sink_udp(0);
        inbound_chan = recv_udp(server_addr.port()) -> map(deserialize_msg) -> tee();

        // Logic
        inbound_chan[0] -> for_each(|m| println!("Got {:?}", m));
        inbound_chan[1] -> map(|EchoMsg { payload, addr }| (EchoResponse { payload, ts: Utc::now() }, addr))
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
