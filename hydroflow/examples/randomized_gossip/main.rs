mod protocol;

use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::Duration;

use chrono::{DateTime, Utc, MIN_DATETIME};
use clap::{Parser, ValueEnum};
use itertools::Itertools;
use hydroflow::hydroflow_syntax;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve};
use tokio::time::sleep;
use rand::random;
use hydroflow_lang::graph::WriteGraphType::Mermaid;
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

    fn user_id(&self) -> &str {
        match self {
            Role::Peer1 => "Peer1",
            Role::Peer2 => "Peer2",
            Role::Peer3 => "Peer3",
            Role::Peer4 => "Peer4",
            Role::Peer5 => "Peer5",
        }
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

    let addr = opts.role.listening_address();
    let (sink, source, addr) = bind_udp_bytes(addr).await;

    let (sender, receiver) = hydroflow::util::unbounded_channel::<Message>();

    clear_screen();

    let mut flow = hydroflow_syntax! {
        outbound = dest_sink_serde(sink);
        inbound = source_stream_serde(source);

        all_peers = source_iter([Role::Peer1, Role::Peer2, Role::Peer3, Role::Peer4, Role::Peer5])
            -> [0]gossip;

        known_messages = union()
            -> fold::<'static>(HashSet::new, |accum: &mut HashSet<ChatMessage>, elem| { accum.insert(elem) })
            -> tee();

        inbound
            -> map(Result::unwrap)
            -> flat_map(|(message, addr)| {
                match message {
                    KnownMessages{ messages } => messages
                }
            })
            -> [0]known_messages;

        known_messages
            -> for_each(|known_messages : HashSet<ChatMessage>| {
                clear_screen();

                let sorted_messages = known_messages.iter()
                    .sorted_by_key(|message| message.ts);

                for message in sorted_messages {
                    println!("{:?}", message);
                }
            });
        known_messages -> [1]trigger_gossip;

        source_stdin()
            -> map(|line| ChatMessage { user_id: opts.role.user_id().to_owned(), message: line.unwrap(), ts: Utc::now() })
            -> [1]known_messages;

        trigger_gossip = zip()
            -> map(|(instant, known_messages)| known_messages )
            -> [1]gossip;

        gossip = cross_join::<'static, 'tick>()
            -> filter_map(|(role, known_messages)| {

                if random() {
                    Some((KnownMessages { messages: known_messages  }, role.listening_address()))
                } else {
                    None
                }

               })
            -> outbound;

        source_interval(Duration::from_secs(1)) -> [0]trigger_gossip;

    };


    // let serde_graph = flow
    //     .meta_graph()
    //     .expect("No graph found, maybe failed to parse.");
    // serde_graph.open_graph(Mermaid, None).unwrap();

    flow.run_async().await.unwrap();
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}
// println!("\x1B[2J\x1B[1;1H{:?}", known_messages)
