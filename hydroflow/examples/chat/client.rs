use chrono::prelude::*;
use colored::Colorize;

use crate::protocol::{deserialize_msg, serialize_msg, Message};
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
        outbound_chan = merge() -> sink_async(outbound);
        inbound_chan = recv_stream(inbound) -> map(|r| deserialize_msg(r)) -> tee();
        connect_acks = inbound_chan[0] -> filter_map(|m: Message| match m {
            Message::ConnectResponse => Some(m),
            _ => None }) -> tee();
        messages = inbound_chan[1] -> filter_map(|m: Message| match m {
            Message::ChatMessage{ nickname, message, ts } =>
                Some(Message::ChatMessage{nickname: nickname, message: message, ts: ts}),
            _ => None });

        // send a single connection request on startup
        recv_iter([()]) -> map(|_m| (serialize_msg(Message::ConnectRequest {
            nickname: opts.name.clone(),
            addr: client_addr,
        }), server_addr)) -> [0]outbound_chan;

        // take stdin and send to server as a msg
        // the join serves to postpone msgs until the connection request is acked
        msg_send = join()
          -> map(|((), (msg, ()))| (msg, server_addr))
          -> [1]outbound_chan;
        lines = recv_stream(stdin_lines)
          -> map(|l: Result<std::string::String, std::io::Error>| l.unwrap())
          -> map(|l| ((), serialize_msg(Message::ChatMessage {nickname: opts.name.clone(), message: l, ts: Utc::now()})))
          -> [0]msg_send;

        // receive and print messages
        messages -> for_each(|m: Message| match m {
            Message::ChatMessage{ nickname, message, ts } => {
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
                ()
        }
        _ => ()
        });

        // handle connect ack
        connect_acks[0] -> for_each(|m: Message| println!("connected: {:?}", m));
        connect_acks[1] -> filter_map(|m: Message| match m {
            Message::ConnectResponse => Some(((), ())),
            _ => None
        }) -> [1]msg_send;

    };

    if let Some(graph) = opts.graph {
        match graph {
            GraphType::Mermaid => {
                println!("{}", hf.generate_mermaid())
            }
            GraphType::Dot => {
                println!("{}", hf.generate_dot())
            }
            GraphType::Json => {
                println!("{}", hf.generate_json())
            }
        }
    }
    hf.run_async().await.unwrap();
}
