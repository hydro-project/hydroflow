use std::net::SocketAddr;
use tokio::net::UdpSocket;

use crate::protocol::{EchoMsg, EchoResponse};
use crate::{GraphType, Opts};
use hydroflow::hydroflow_syntax;
use hydroflow::util::{deserialize_msg, serialize_msg};

pub(crate) async fn run_client(opts: Opts, server_addr: SocketAddr, my_addr: SocketAddr) {
    println!("Attempting to connect to server at {}", server_addr);

    println!("{:?} live!", opts.role);

    let mut flow = hydroflow_syntax! {
        // set up channels
        outbound_chan = map(|(m,a)| (serialize_msg(m), a)) -> sink_udp(0);
        inbound_chan = recv_udp(my_addr.port()) -> map(deserialize_msg);

        // take stdin and send to server as an Echo::Message
        lines = recv_stdin() -> map(|l| (EchoMsg{ payload: l.unwrap(), addr: my_addr, }, server_addr))
            -> outbound_chan;

        // receive and print messages
        inbound_chan[msgs] -> for_each(|m: EchoResponse| println!("{:?}", m));
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
