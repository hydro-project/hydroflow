use chrono::prelude::*;
use colored::Colorize;
use hydroflow::hydroflow_syntax;
use hydroflow::util::{UdpSink, UdpStream};

use crate::protocol::Message;
use crate::{GraphType, Opts};

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

pub(crate) async fn run_client(outbound: UdpSink, inbound: UdpStream, opts: Opts) {
    // server_addr is required for client
    let server_addr = opts.server_addr.expect("Client requires a server address");
    println!("Client live!");

    let mut hf = hydroflow_syntax! {
        // set up channels
        outbound_chan = union() -> dest_sink_serde(outbound);
        inbound_chan = source_stream_serde(inbound) -> map(Result::unwrap) -> map(|(m, _)| m)
            ->  demux(|m, var_args!(acks, msgs, errs)|
                    match m {
                        Message::ConnectResponse => acks.give(m),
                        Message::ChatMsg {..} => msgs.give(m),
                        _ => errs.give(m),
                    }
                );
        inbound_chan[errs] -> for_each(|m| println!("Received unexpected message type: {:?}", m));

        // send a single connection request on startup
        initialize() -> map(|_m| (Message::ConnectRequest, server_addr)) -> [0]outbound_chan;

        // take stdin and send to server as a msg
        // the batch serves to buffer msgs until the connection request is acked
        lines = source_stdin()
          -> map(|l| Message::ChatMsg {
                    nickname: opts.name.clone(),
                    message: l.unwrap(),
                    ts: Utc::now()})
          -> [input]msg_send;
        inbound_chan[acks] -> persist() -> [signal]msg_send;
        msg_send = defer_signal() -> map(|msg| (msg, server_addr)) -> [1]outbound_chan;

        // receive and print messages
        inbound_chan[msgs] -> for_each(pretty_print_msg);
    };

    // optionally print the dataflow graph
    if let Some(graph) = opts.graph {
        let serde_graph = hf
            .meta_graph()
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
            }
        }
    }

    hf.run_async().await.unwrap();
}
