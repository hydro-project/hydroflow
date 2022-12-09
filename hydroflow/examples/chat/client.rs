use chrono::prelude::*;
use colored::Colorize;

use crate::protocol::Message;
use crate::GraphType;
use chrono::Utc;
use hydroflow::hydroflow_syntax;
use hydroflow::pusherator::Pusherator;
use hydroflow::util::{UdpSink, UdpStream};
use std::net::SocketAddr;

fn pretty_print_msg(msg: Message) {
    if let Message::ChatMsg {
        nickname,
        message,
        ts,
    } = msg
    {
        println!(
            "{} {}: {}",
            ts.with_timezone(&Local)
                .format("%b %-d, %-I:%M:%S")
                .to_string()
                .truecolor(126, 126, 126)
                .italic(),
            nickname.green().italic(),
            message,
        );
    }
}

pub(crate) async fn run_client(
    outbound: UdpSink,
    inbound: UdpStream,
    server_addr: SocketAddr,
    name: String,
    graph: Option<GraphType>,
) {
    println!("Client live!");

    let mut hf = hydroflow_syntax! {
        // set up channels
        outbound_chan = merge() -> sink_async_serde(outbound);
        inbound_chan = recv_stream_serde(inbound) -> map(|(m, _)| m)
            ->  demux(|m, tl!(acks, msgs, errs)|
                    match m {
                        Message::ConnectResponse => acks.give(m),
                        Message::ChatMsg {..} => msgs.give(m),
                        _ => errs.give(m),
                    }
                );
        inbound_chan[errs] -> for_each(|m| println!("Received unexpected message type: {:?}", m));

        // send a single connection request on startup
        recv_iter([()]) -> map(|_m| (Message::ConnectRequest, server_addr)) -> [0]outbound_chan;

        // take stdin and send to server as a msg
        // the join serves to buffer msgs until the connection request is acked
        msg_send = cross_join() -> map(|(msg, _)| (msg, server_addr)) -> [1]outbound_chan;
        lines = recv_stdin()
          -> map(|l| Message::ChatMsg {
                    nickname: name.clone(),
                    message: l.unwrap(),
                    ts: Utc::now()})
          -> [0]msg_send;

        // receive and print messages
        inbound_chan[msgs] -> for_each(|m: Message| pretty_print_msg(m));

        // handle connect ack
        inbound_chan[acks] -> [1]msg_send;

    };

    if let Some(graph) = graph {
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
