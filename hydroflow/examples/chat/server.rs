use crate::{GraphType, Opts};

use crate::protocol::{MemberRequest, MemberResponse};

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use serde_json::json;
use tokio::net::UdpSocket;

pub(crate) async fn run_server(opts: Opts) {
    // Set up sockets.
    // First, use the canonical ip:port passed into the opts for membership requests
    let server_members_socket = UdpSocket::bind((opts.addr.clone(), opts.port))
        .await
        .unwrap();
    let (server_members_send, server_members_recv) =
        hydroflow::util::udp_lines(server_members_socket);

    // Second, allocate a new port from the OS for messaging
    let server_msg_socket = UdpSocket::bind((opts.addr.clone(), 0)).await.unwrap();
    // Because we requested port 0, we got assigned an arbitrary port. Read that back into a SocketAddr
    let server_msg_addr = server_msg_socket.local_addr().unwrap();
    let (server_msg_send, server_msg_recv) = hydroflow::util::udp_lines(server_msg_socket);

    println!("Server live!");

    let mut df: Hydroflow = hydroflow_syntax! {
        // Handle member requests. Respond with the ip:port we allocated for messages.
        members = recv_stream(server_members_recv)
            -> map(|r| serde_json::from_str(&(r.unwrap().0)).unwrap())
            -> tee();
        members[0] -> for_each(|m: MemberRequest| println!("got member: {:?}", m));
        members[1] -> map(|m: MemberRequest|
                          (json!(MemberResponse{messages_addr: server_msg_addr}).to_string(),
                           m.connect_addr))
            -> sink_async(server_members_send);

             // Handle messages.
        // Every message that comes in will be joined with every member seen.
        // The result is that each member will see all messages (even from the past).
        broadcast = join()
            -> map(|((), (msg, addr))| (msg.to_owned(), addr))
            -> sink_async(server_msg_send);
        // Left branch of the join is the message stream
        recv = recv_stream(server_msg_recv)
            -> map(|r| r.unwrap())
            -> tee();
        recv[0] -> for_each(|m| println!("got msg: {:?}", m));
        recv[1] -> map(|(msg, _addr): (String, std::net::SocketAddr)| ((), msg)) -> [0]broadcast;
        // Right branch of the join is the stream of members
        members[2] -> map(|m: MemberRequest| ((), m.messages_addr)) -> [1]broadcast;
    };

    match opts.graph {
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

    df.run_async().await.unwrap();
}
