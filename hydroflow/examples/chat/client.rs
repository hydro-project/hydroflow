use chrono::prelude::*;
use colored::Colorize;

use crate::helpers::{deserialize_msg, is_chat_msg, is_connect_resp, serialize_msg};
use crate::protocol::Message;
use crate::{GraphType, Opts};
use chrono::Utc;
use hydroflow::hydroflow_syntax;
use std::net::SocketAddr;
use tokio::io::AsyncBufReadExt;
use tokio::net::UdpSocket;
use tokio_stream::wrappers::LinesStream;

pub(crate) async fn run_client(opts: Opts) {
    // set up network and I/O channels
    let server_addr: SocketAddr = format!("{}:{}", opts.addr, opts.port).parse().unwrap();

    let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let client_addr = client_socket.local_addr().unwrap();
    let (outbound, inbound) = hydroflow::util::udp_lines(client_socket);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let stdin_lines = LinesStream::new(reader.lines());
    println!("Client live!");

    let mut hf = hydroflow_syntax! {
        // set up channels
        outbound_chan = merge() -> map(|(m,a)| (serialize_msg(m), a)) -> sink_async(outbound);
        inbound_chan = recv_stream(inbound) -> map(deserialize_msg) -> tee();
        connect_acks = inbound_chan[0] -> filter_map(is_connect_resp) -> tee();
        messages = inbound_chan[1] -> filter_map(is_chat_msg);

        // send a single connection request on startup
        recv_iter([()]) -> map(|_m| (Message::ConnectRequest {
            nickname: opts.name.clone(),
            addr: client_addr,
        }, server_addr)) -> [0]outbound_chan;

        // take stdin and send to server as a msg
        // the join serves to postpone msgs until the connection request is acked
        msg_send = cross_join() -> map(|(msg, _)| (msg, server_addr)) -> [1]outbound_chan;
        lines = recv_stream(stdin_lines)
          -> map(|l| Message::ChatMsg {
                    nickname: opts.name.clone(),
                    message: l.unwrap(),
                    ts: Utc::now()})
          -> [0]msg_send;

        // receive and print messages
        messages -> for_each(|m: Message| if let Message::ChatMsg{ nickname, message, ts } = m {
                println!(
                    "{} {}: {}",
                    ts
                        .with_timezone(&Local)
                        .format("%b %-d, %-I:%M:%S")
                        .to_string()
                        .truecolor(126, 126, 126)
                        .italic(),
                    nickname.green().italic(),
                    message,
                );
        });

        // handle connect ack
        connect_acks[0] -> for_each(|m: Message| println!("connected: {:?}", m));
        connect_acks[1] -> filter_map(is_connect_resp) -> [1]msg_send;

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
