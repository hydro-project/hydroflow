mod protocol;

use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::Duration;

use chrono::{DateTime, Utc, MIN_DATETIME};
use clap::{Parser, ValueEnum};
use hydroflow::hydroflow_syntax;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve};
use tokio::time::sleep;
use rand::random;
use crate::protocol::Message::KnownMessages;
use crate::protocol::{ChatMessage, Message};

#[derive(Clone, ValueEnum, Debug, Eq, PartialEq)]
enum Role {
    Peer1,
    Peer2,
    Peer3,
    Peer4,
    Peer5,
}

impl Role {
    fn listening_address(&self) -> SocketAddr {
        match self {
            // TODO: Cleanup consts.
            Role::Peer1 => ipv4_resolve("localhost:54321"),
            Role::Peer2 => ipv4_resolve("localhost:54322"),
            Role::Peer3 => ipv4_resolve("localhost:54323"),
            Role::Peer4 => ipv4_resolve("localhost:54324"),
            Role::Peer5 => ipv4_resolve("localhost:54325"),
        }
        .unwrap()
    }
}

/// TODO: Help text coming soon.
#[derive(Parser)]
struct Opts {
    /// TODO: Help text coming soon.
    #[clap(value_enum, long)]
    role: Role,
}

#[hydroflow::main]
async fn main() {
    let opts = Opts::parse();

    // let addr = opts.role.listening_address();
    // let (_outbound, _inbound, addr) = bind_udp_bytes(addr).await;

    // println!("Listening on {:?}", addr);
    let (sender, receiver) = hydroflow::util::unbounded_channel::<Message>();

    let mut flow = hydroflow_syntax! {
        // outbound_chan = dest_sink_serde(outbound);
        // inbound_chan = source_stream_serde(inbound);

        all_peers = source_iter([Role::Peer1, Role::Peer2, Role::Peer3, Role::Peer4, Role::Peer5])
            -> [0]gossip;

        known_messages = union()
            -> fold::<'static>(HashSet::new, |accum: &mut HashSet<ChatMessage>, elem| { accum.insert(elem) })
            -> [1]trigger_gossip;

        source_stream(receiver)
            -> flat_map(|message| {
                match message {
                    KnownMessages{ messages } => messages
                }
            })
            -> [0]known_messages;

        source_stdin()
            -> map(|line| ChatMessage { user_id: "rohit".to_owned(), message: line.unwrap(), ts: Utc::now() })
            -> [1]known_messages;

        trigger_gossip = zip()
            -> map(|(instant, known_messages)| known_messages )
            -> [1]gossip;

        gossip = cross_join::<'static, 'tick>()
            -> filter_map(|(role, known_messages)| {

                if random() {
                    Some((known_messages, role.listening_address()))
                } else {
                    None
                }

               })
            -> for_each(|(role, known_messages)| println!("{:?}", known_messages));

        source_interval(Duration::from_secs(1)) -> [0]trigger_gossip;

    };

    flow.run_async().await.unwrap();
}
// println!("\x1B[2J\x1B[1;1H{:?}", known_messages)
