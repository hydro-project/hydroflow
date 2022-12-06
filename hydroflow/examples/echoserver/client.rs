use crate::helpers::{deserialize_msg, resolve_ipv4_connection_addr, serialize_msg};
use crate::protocol::{EchoMsg, EchoResponse};
use crate::{GraphType, Opts};
use hydroflow::hydroflow_syntax;
use textnonce::TextNonce;
use tokio::io::AsyncBufReadExt;
use tokio::net::UdpSocket;
use tokio_stream::wrappers::LinesStream;

pub(crate) async fn run_client(opts: Opts) {
    // set up network and I/O channels
    let server_ip = opts
        .server_addr
        .expect("Clients must specify --server-addr");
    let server_port = opts
        .server_port
        .expect("Clients must specify --server-port");

    let server_addr = resolve_ipv4_connection_addr(server_ip, server_port)
        .expect("Unable to resolve server address");
    println!("Attempting to connect to server at {}", server_addr);

    let client_addr = resolve_ipv4_connection_addr(opts.addr, opts.port)
        .expect("Unable to resolve client address");
    let client_socket = UdpSocket::bind(client_addr).await.unwrap();
    println!("{:?} is bound to {}", opts.role, client_addr);
    let (outbound, inbound) = hydroflow::util::udp_lines(client_socket);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let stdin_lines = LinesStream::new(reader.lines());

    println!("{:?} live!", opts.role);

    let mut flow = hydroflow_syntax! {
        // set up channels
        outbound_chan = map(|(m,a)| (serialize_msg(m), a)) -> sink_async(outbound);
        inbound_chan = recv_stream(inbound) -> map(deserialize_msg);

        // take stdin and send to server as an Echo::Message
        lines = recv_stream(stdin_lines)
          -> map(|l|    (EchoMsg{
                            nonce: TextNonce::new().to_string(),
                            payload: l.unwrap(),
                            addr: client_addr},
                        server_addr))
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
