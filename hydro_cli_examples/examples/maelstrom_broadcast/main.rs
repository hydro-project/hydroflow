use std::collections::HashMap;
use std::io::Result;

use hydroflow::bytes::{Bytes, BytesMut};
use hydroflow::hydroflow_syntax;
use hydroflow::util::cli::{
    ConnectedDemux, ConnectedDirect, ConnectedSink, ConnectedSource, ConnectedTagged,
};
use hydroflow::util::serialize_to_bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Broadcast {
    pub msg_id: Value,
    pub message: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BroadcastOk {
    pub in_reply_to: Value,
}

impl Broadcast {
    pub fn respond(self) -> BroadcastOk {
        BroadcastOk {
            in_reply_to: self.msg_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Topology {
    pub msg_id: Value,
    pub topology: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopologyOk {
    pub in_reply_to: Value,
}

impl Topology {
    pub fn respond(self) -> TopologyOk {
        TopologyOk {
            in_reply_to: self.msg_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Read {
    pub msg_id: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReadOk {
    pub in_reply_to: Value,
    pub messages: Vec<usize>,
}

impl Read {
    pub fn respond(self, messages: Vec<usize>) -> ReadOk {
        ReadOk {
            in_reply_to: self.msg_id,
            messages,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Gossip {
    pub messages: Vec<usize>,
}

fn parse_input<T: DeserializeOwned>(bytes: Result<BytesMut>) -> T {
    let bytes = bytes.unwrap().to_vec();

    let string = String::from_utf8(bytes).unwrap();

    serde_json::from_str::<T>(&string).unwrap()
}

fn prep_output<T: Serialize>(output: T) -> Bytes {
    let string = serde_json::to_string(&output).unwrap();
    serialize_to_bytes(string)
}

fn parse_tagged_input<T: DeserializeOwned>(bytes: Result<(u32, BytesMut)>) -> (u32, T) {
    let bytes = bytes.unwrap();
    let string = String::from_utf8(bytes.1.to_vec()).unwrap();
    (bytes.0, serde_json::from_str::<T>(&string).unwrap())
}

fn prep_demux_output<T: Serialize>((peer, state): (usize, T)) -> (u32, Bytes) {
    let output = serde_json::to_string(&state).unwrap();
    (peer as u32, serialize_to_bytes(output))
}

#[hydroflow::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let node_id = ports.node_id;
    let node_name = ports.node_names.get(&node_id).unwrap().clone();
    // map from node name to id
    let node_ids: HashMap<String, usize> = ports
        .node_names
        .iter()
        .map(|(key, value)| (value.clone(), *key))
        .collect();

    let broadcast_in = ports
        .port("broadcast_in")
        .connect::<ConnectedDirect>()
        .await
        .into_source();
    let broadcastok_out = ports
        .port("broadcastok_out")
        .connect::<ConnectedDirect>()
        .await
        .into_sink();

    let topology_in = ports
        .port("topology_in")
        .connect::<ConnectedDirect>()
        .await
        .into_source();
    let topologyok_out = ports
        .port("topologyok_out")
        .connect::<ConnectedDirect>()
        .await
        .into_sink();

    let read_in = ports
        .port("read_in")
        .connect::<ConnectedDirect>()
        .await
        .into_source();
    let readok_out = ports
        .port("readok_out")
        .connect::<ConnectedDirect>()
        .await
        .into_sink();

    // This port is does not transfer maelstrom payloads, but rather passes custom payloads between nodes
    let gossip_out = ports
        .port("gossip_out")
        .connect::<ConnectedDemux<ConnectedDirect>>()
        .await
        .into_sink();
    let gossip_in = ports
        .port("gossip_in")
        .connect::<ConnectedTagged<ConnectedDirect>>()
        .await
        .into_source();

    let df = hydroflow_syntax! {
        broadcast = source_stream(broadcast_in) -> map(parse_input) -> tee();
        topology = source_stream(topology_in)-> map(parse_input) -> tee();
        read = source_stream(read_in) -> map(parse_input::<Read>);

        broadcastok = map(prep_output) -> dest_sink(broadcastok_out);
        topologyok = map(prep_output) -> dest_sink(topologyok_out);
        readok = map(prep_output) -> dest_sink(readok_out);

        gossip = source_stream(gossip_in) -> map(parse_tagged_input::<usize>);
        gossipout = map(prep_demux_output) -> dest_sink(gossip_out);

        // Ack topology message from maelstrom
        topology -> map(|top: Topology| top.respond()) -> topologyok;
        // Ack all broadcasts
        broadcast -> map(|b: Broadcast| b.respond()) -> broadcastok;

        // Identifies all neighbors of the current node
        topology -> flat_map(|top| top.topology.get(&node_name).unwrap().clone()) -> map(|node_name| *node_ids.get(&node_name).unwrap()) -> [0]forwards;


        // Set of all gossip messages
        gossip_message = gossip -> map(|(_, message)| message) -> tee();

        // Gossip messages should be gossiped further
        gossip_message -> forward_message;

        // Gossip messages should be added to the message list
        gossip_message -> messages;


        // Join the stream of adjacent nodes with the new messages to gossip and output
        forwards = cross_join::<'static, 'tick>() -> gossipout;

        // Set of messages to gossip to neighbors
        forward_message = union() -> unique::<'static>() -> [1]forwards;

        // All new messages from maelstrom should be gossipped
        message -> forward_message;

        // Broadcast messages
        message = broadcast -> map(|b: Broadcast| b.message) -> tee();

        // Messages is singleton list of all unique messages recieved
        messages = union() -> unique::<'static>() -> fold::<'static>(Vec::new, |accum: &mut Vec<usize>, elem| {accum.push(elem)}) -> [1]output;
        message -> messages;

        // When we get a read, trigger a readok containing the messages singleton
        read -> [0]output;
        output = cross_join() -> map(|(read, messages)| read.respond(messages)) -> readok;
    };

    hydroflow::util::cli::launch_flow(df).await;
}
