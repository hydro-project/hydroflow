use crate::{GraphType, Opts};

use crate::protocol::{deserialize_msg, serialize_msg, Message};

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use tokio::net::UdpSocket;

pub(crate) async fn run_server(opts: Opts) {
    // First, set up the socket
    let server_socket = UdpSocket::bind(("127.0.0.1", opts.port)).await.unwrap();
    let (outbound, inbound) = hydroflow::util::udp_lines(server_socket);
    println!("Server live!");

    let mut df: Hydroflow = hydroflow_syntax! {
        // set up channels
        outbound_chan = merge() -> sink_async(outbound);
        inbound_chan = recv_stream(inbound) -> map(|m| deserialize_msg(m)) -> tee();
        member_chan = inbound_chan[0] -> filter_map(|m: Message| match m {
                Message::ConnectRequest { nickname: _, addr: _ } => Some(m),
                _ => None });
        members = member_chan -> tee();

        msg_chan = inbound_chan[1] -> filter_map(|m: Message| match m {
                Message::ChatMessage{ nickname, message, ts } =>
                    Some(Message::ChatMessage{nickname: nickname, message: message, ts: ts}),
                _ => None });

        // Member request handler. Respond with the ip:port we allocated for messages.
        members[0] -> filter_map(|m: Message| {
                    match m {
                        Message::ConnectRequest { nickname: _, addr } =>
                            Some((serialize_msg(Message::ConnectResponse), addr)),
                        _ => None
                    }
                } ) -> [0]outbound_chan;

        // Message handler.
        // Every message that comes in will be joined with every member seen.
        // Each member will see all messages (even from the past).
        broadcast = join()
            -> map(|((), (msg, addr))| (msg.to_owned(), addr))
            -> [1]outbound_chan;
        // Left branch of the join is the message stream
        msg_chan -> map(|m| ((), serialize_msg(m))) -> [0]broadcast;
        // Right branch of the join is the stream of members
        members[1] -> filter_map(|m: Message| {
            match m {
                Message::ConnectRequest{ nickname: _, addr } =>
                    Some(((), addr)),
                _ => None
            }
        }) -> [1]broadcast;
    };

    if let Some(graph) = opts.graph {
        match graph {
            GraphType::Mermaid => {
                println!("{}", df.generate_mermaid())
            }
            GraphType::Dot => {
                println!("{}", df.generate_dot())
            }
            GraphType::Json => {
                println!("{}", df.generate_json())
            }
        }
    }

    df.run_async().await.unwrap();
}
