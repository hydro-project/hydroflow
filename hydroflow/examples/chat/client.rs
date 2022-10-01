use chrono::prelude::*;
use colored::Colorize;

use crate::protocol::{ChatMessage, MemberRequest, MemberResponse};
use crate::{GraphType, Opts};
use chrono::Utc;
use hydroflow::hydroflow_syntax;
use serde_json::json;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use tokio::io::AsyncBufReadExt;
use tokio::net::UdpSocket;
use tokio_stream::wrappers::LinesStream;

pub(crate) async fn run_client(opts: Opts) {
    let addr = IpAddr::from_str(&opts.addr).unwrap();
    let server_members_addr: SocketAddr = format!("{}:{}", opts.addr, opts.port).parse().unwrap();

    let client_msg_socket = UdpSocket::bind((addr, 0)).await.unwrap();
    let client_msg_addr = client_msg_socket.local_addr().unwrap();
    let (client_msg_send, client_msg_recv) = hydroflow::util::udp_lines(client_msg_socket);
    let client_members_socket = UdpSocket::bind((addr, 0)).await.unwrap();
    let client_members_addr = client_members_socket.local_addr().unwrap();
    let (client_members_send, client_members_recv) =
        hydroflow::util::udp_lines(client_members_socket);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let stdin_lines = LinesStream::new(reader.lines());
    println!("Client live!");
    let member = json!(MemberRequest {
        nickname: opts.name.clone(),
        connect_addr: client_members_addr,
        messages_addr: client_msg_addr,
    })
    .to_string();

    let mut hf = hydroflow_syntax! {
        // send a members message
        recv_iter([()]) -> map(|_m| (member.clone(), server_members_addr)) -> sink_async(client_members_send);

        // send the messages
        msg_send = join()
          -> map(|((), (msg, addr)): ((), (std::string::String, SocketAddr))| (msg, addr))
          -> sink_async(client_msg_send);
        lines = recv_stream(stdin_lines)
          -> map(|l: Result<std::string::String, std::io::Error>| l.unwrap())
          -> map(|l| ((), json!(ChatMessage {nickname: opts.name.clone(), message: l, ts: Utc::now()}).to_string()))
          -> [0]msg_send;

        // receive and print messages
        recv_stream(client_msg_recv)
           -> map(|m| m.unwrap())
           -> map(|(m, _addr)| serde_json::from_str(&m).unwrap())
           -> for_each(|m: ChatMessage| println!(
            "{} {}: {}",
            m.ts
                .with_timezone(&Local)
                .format("%b %-d, %-I:%M:%S")
                .to_string()
                .truecolor(126, 126, 126)
                .italic(),
            m.nickname.green().italic(),
            m.message,
        ));

        // handle connect ack, which contains the messages port
        connect_ack = recv_stream(client_members_recv)
          -> map(|r| serde_json::from_str(&(r.unwrap().0)).unwrap()) -> tee();
        connect_ack[0] -> map(|m: MemberResponse| ((), m.messages_addr)) -> [1]msg_send;
        connect_ack[1] -> for_each(|m: MemberResponse| println!("connection acked: {:?}", m));
    };

    match opts.graph {
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
    hf.run_async().await.unwrap();
}
