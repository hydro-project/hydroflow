use crate::helpers::{
    deserialize_msg, is_resp, parse_command, resolve_ipv4_connection_addr, serialize_msg,
};
use crate::{GraphType, Opts};
use hydroflow::hydroflow_syntax;
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

    println!("Client is bound to {}", client_addr);

    let (outbound, inbound) = hydroflow::util::udp_lines(client_socket);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let stdin_lines = LinesStream::new(reader.lines());
    println!("Client live!");

    let mut hf = hydroflow_syntax! {
        // set up channels
        outbound_chan = map(|(m,a)| (serialize_msg(m), a)) -> sink_async(outbound);
        inbound_chan = recv_stream(inbound) -> map(deserialize_msg);
        resps = inbound_chan -> filter_map(is_resp);

        // read in commands from stdin and forward to server
        recv_stream(stdin_lines)
            -> filter_map(|line| parse_command(line.unwrap(), client_addr))
            -> map(|msg| { (msg, server_addr) })
            -> [0]outbound_chan;

        // print inbound msgs
        resps -> for_each(|m| println!("Got a Response: {:?}", m));

        // // take stdin and send to server as a msg
        // // the join serves to postpone msgs until the connection request is acked
        // lines = recv_stream(stdin_lines)
        //   -> map(|l| KVSMessage::ChatMsg {
        //             nickname: opts.name.clone(),
        //             message: l.unwrap(),
        //             ts: Utc::now()}) -> null();

        // // receive and print messages
        // messages -> for_each(|m: KVSMessage| if let KVSMessage::ChatMsg{ nickname, message, ts } = m {
        //         println!(
        //             "{} {}: {}",
        //             ts
        //                 .with_timezone(&Local)
        //                 .format("%b %-d, %-I:%M:%S")
        //                 .to_string()
        //                 .truecolor(126, 126, 126)
        //                 .italic(),
        //             nickname.green().italic(),
        //             message,
        //         );
        // });

    };

    if let Some(graph) = opts.graph {
        let serde_graph = hf
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
    hf.run_async().await.unwrap();
}
