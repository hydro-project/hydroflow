use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::Duration;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve};
use hydroflow_macro::hydroflow_syntax;

use crate::{default_server_address, Opts, Role};
use crate::protocol::{Message, MessageWithAddr};
use crate::protocol::Message::ChatMsg;
use crate::Role::{GossipingServer1, GossipingServer2, GossipingServer3, GossipingServer4, GossipingServer5, Client, Server};


#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug, Hash)]
pub struct ChatMessage {
    nickname: String,
    message: String,
    ts: DateTime<Utc>,
}

enum InfectionOperation {
    InfectWithMessage { msg: ChatMessage },
    RemoveForMessage { msg: ChatMessage },
}

const REMOVAL_PROBABILITY: f32 = 1.0 / 2.0;

fn gossip_address(role: &Role) -> SocketAddr {
    match role {
        Client | Server => { panic!("Incorrect role {:?} for gossip server.", role) }
        GossipingServer1 => ipv4_resolve("localhost:54322"),
        GossipingServer2 => ipv4_resolve("localhost:54323"),
        GossipingServer3 => ipv4_resolve("localhost:54324"),
        GossipingServer4 => ipv4_resolve("localhost:54325"),
        GossipingServer5 => ipv4_resolve("localhost:54326"),
    }.unwrap()
}

pub(crate) async fn run_gossiping_server(opts: Opts) {
    // If a server address & port are provided as command-line inputs, use those, else use the
    // default.
    let server_address = opts
        .address
        .unwrap_or_else(|| default_server_address());

    let all_members = vec![GossipingServer1,
                           GossipingServer2,
                           GossipingServer3,
                           GossipingServer4,
                           GossipingServer4,
                           GossipingServer5];

    let other_members : Vec<Role> = all_members.iter().filter(|role| **role != opts.role)
        .cloned()
        .collect();

    let gossip_listening_addr = gossip_address(&opts.role);

    println!("Starting server on {:?}", server_address);

    let (client_outbound, client_inbound, actual_server_addr) = bind_udp_bytes(server_address).await;
    let (gossip_outbound, gossip_inbound, _) = bind_udp_bytes(gossip_listening_addr).await;

    println!("Server is live! Listening on {:?}. Gossiping On: {:?}", actual_server_addr, gossip_listening_addr);
    let mut hf: Hydroflow = hydroflow_syntax! {
        // Define shared inbound and outbound channels
        client_out = union() -> dest_sink_serde(client_outbound);
        client_in = source_stream_serde(client_inbound)
            -> map(Result::unwrap)
            -> map(|(msg, addr)| MessageWithAddr::from_message(msg, addr))
            -> demux_enum::<MessageWithAddr>();
        clients = client_in[ConnectRequest] -> map(|(addr,)| addr) -> tee();
        client_in[ConnectResponse] -> for_each(|(addr,)| println!("Received unexpected `ConnectResponse` as server from addr {}.", addr));

        // Pipeline 1: Acknowledge client connections
        clients[0] -> map(|addr| (Message::ConnectResponse, addr)) -> [0]client_out;

        // Pipeline 2: Broadcast messages to all clients and gossip to other servers
        messages_from_connected_client = client_in[ChatMsg]
            -> map(|(_addr, nickname, message, ts)| ChatMessage { nickname, message, ts })
            -> tee();

        messages_from_connected_client[0] -> maybe_new_messages;

        clients[1] -> [1]broadcast;
        broadcast = cross_join::<'tick, 'static>() -> [1]client_out;

        // Pipeline 3: Gossip-based broadcast to other servers.
        gossip_out = dest_sink_serde(gossip_outbound);
        gossip_in = source_stream_serde(gossip_inbound)
            -> map(Result::unwrap)
            -> map(|(message, _)| message)
            -> maybe_new_messages;

        // If you think there may be a new message, send it here.
        maybe_new_messages = union();

        // Actually new message is a stream of messages that have never been seen before.
        actually_new_messages = difference() -> tee();
        maybe_new_messages -> [pos]actually_new_messages;
        all_messages -> [neg]actually_new_messages;

        // When we have a new message, we should do 3 things
        // 1. Broadcast it to the clients connected locally.
        // 2. Add it to the set of known messages.
        // 3. Add it to the set of messages currently infecting this server.
        actually_new_messages -> defer_tick() -> all_messages; // Add to known messages
        actually_new_messages
            -> map(|chat_msg: ChatMessage| Message::ChatMsg {
                    nickname: chat_msg.nickname,
                    message: chat_msg.message,
                    ts: chat_msg.ts})
            -> [0]broadcast;
        actually_new_messages
            -> map(|msg: ChatMessage| InfectionOperation::InfectWithMessage { msg })
            -> infecting_messages;

        // Holds all the known messages
        all_messages = fold::<'static>(HashSet::<ChatMessage>::new, |accum, message| {
            accum.insert(message)
        }) -> flatten();

        // Holds a set of messages that are currently infecting this server
        infecting_messages = union() -> fold::<'static>(HashSet::<ChatMessage>::new, |accum, op| {
            match op {
                InfectionOperation::InfectWithMessage{ msg } => {accum.insert(msg)},
                InfectionOperation::RemoveForMessage{ msg } => { accum.remove(&msg) }
            }
        }) -> flatten() -> tee();

        // Infection process.
        // Every 1 second, the infecting messages are dispatched to a randomly selected peer. They
        // are blindly removed with a 1/K probability after this.
        triggered_messages = cross_join();
        source_interval(Duration::from_secs(1)) -> [0]triggered_messages; // The time trigger to perform a round of gossip
        infecting_messages -> [1]triggered_messages;

        triggered_messages
            -> map(|(_, message)| {
                    // Choose a random peer
                    let random_peer = other_members.choose(&mut thread_rng()).unwrap();
                    (message, gossip_address(random_peer))
               })
            -> gossip_out; // Strip the timestamp and assign a peer to send to.

        // Removal process
        infecting_messages
            -> filter_map(|msg| {
                if (rand::random::<f32>() < REMOVAL_PROBABILITY) {
                    Some(InfectionOperation::RemoveForMessage{ msg })
                } else {
                    None
                }
            })
            -> infecting_messages;
    };

    if let Some(graph) = opts.graph {
        let serde_graph = hf
            .meta_graph()
            .expect("No graph found, maybe failed to parse.");
        serde_graph.open_graph(graph, opts.write_config).unwrap();
    }

    hf.run_async().await.unwrap();
}
