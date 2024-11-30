use std::collections::HashSet;
use std::net::SocketAddr;

use hydroflow_macro::DemuxEnum;
use lattices::map_union::{MapUnionBTreeMap, MapUnionHashMap};
// use lattices::map_union_with_tombstones::
use lattices::set_union::SetUnionHashSet;
use lattices::{DomPair, IsBot, LatticeFrom, Max, Merge};
use serde::{Deserialize, Serialize};

pub type Key = String;
pub type Value = String;
pub type NodeId = SocketAddr;

pub type VClock = MapUnionHashMap<NodeId, Max<usize>>;
// pub type Anna = MapUnionHashMap<Key, DomPair<VClock, SetUnionHashSet<Value>>>;

pub type Anna = MapUnionHashMap<Key, DomPair<VClock, AnnaValue>>;

#[derive(Clone, Debug)]
pub enum AnnaValue {
    Value(SetUnionHashSet<String>),
    HashRing(MapUnionBTreeMap<u64, SetUnionHashSet<NodeId>>),
}

impl Merge<AnnaValue> for AnnaValue {
    fn merge(&mut self, other: AnnaValue) -> bool {
        match (self, other) {
            (AnnaValue::Value(x), AnnaValue::Value(y)) => x.merge(y),
            (AnnaValue::HashRing(x), AnnaValue::HashRing(y)) => x.merge(y),
            _ => panic!(),
        }
    }
}

impl IsBot for AnnaValue {
    fn is_bot(&self) -> bool {
        match self {
            AnnaValue::Value(x) => x.is_bot(),
            AnnaValue::HashRing(x) => x.is_bot(),
        }
    }
}

impl LatticeFrom<AnnaValue> for AnnaValue {
    fn lattice_from(other: AnnaValue) -> Self {
        other
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, DemuxEnum)]
pub enum ServerReq {
    ClientPut { key: String, value: String },
    ClientGet { key: String },
    AddNode { node_id: NodeId },
    RemoveNode { node_id: NodeId },
}

#[derive(Clone, Debug, Serialize, Deserialize, DemuxEnum)]
pub enum ServerReqWithSrc {
    ClientPut {
        src: NodeId,
        key: String,
        value: String,
    },
    ClientGet {
        src: NodeId,
        key: String,
    },
    AddNode {
        src: NodeId,
        node_id: NodeId,
    },
    RemoveNode {
        src: NodeId,
        node_id: NodeId,
    },
}

impl ServerReqWithSrc {
    pub fn from_server_req(server_req: ServerReq, src: NodeId) -> Self {
        match server_req {
            ServerReq::ClientPut { key, value } => Self::ClientPut { src, key, value },
            ServerReq::ClientGet { key } => Self::ClientGet { src, key },
            ServerReq::AddNode { node_id } => Self::AddNode { src, node_id },
            ServerReq::RemoveNode { node_id } => Self::RemoveNode { src, node_id },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, DemuxEnum)]
pub enum ServerResp {
    ServerResponse {
        key: String,
        value: Option<HashSet<String>>,
    },
    NotOwned {
        hash_ring: MapUnionBTreeMap<u64, SetUnionHashSet<NodeId>>,
    },
}

// #[derive(Clone, Debug, Serialize, Deserialize, DemuxEnum)]
// pub enum KvsMessage {
//     ServerResponse { key: String, value: String },
//     ClientPut { key: String, value: String },
//     ClientGet { key: String },
//     PeerGossip { key: String, value: String },
//     PeerJoin,
// }

// #[derive(Clone, Debug, DemuxEnum)]
// pub enum KvsMessageWithAddr {
//     ServerResponse {
//         key: String,
//         value: String,
//         addr: SocketAddr,
//     },
//     ClientPut {
//         key: String,
//         value: String,
//         addr: SocketAddr,
//     },
//     ClientGet {
//         key: String,
//         addr: SocketAddr,
//     },
//     PeerGossip {
//         key: String,
//         value: String,
//         addr: SocketAddr,
//     },
//     PeerJoin {
//         addr: SocketAddr,
//     },
// }
// impl KvsMessageWithAddr {
//     pub fn from_message(message: KvsMessage, addr: SocketAddr) -> Self {
//         match message {
//             KvsMessage::ServerResponse { key, value } => Self::ServerResponse { key, value, addr },
//             KvsMessage::ClientPut { key, value } => Self::ClientPut { key, value, addr },
//             KvsMessage::ClientGet { key } => Self::ClientGet { key, addr },
//             KvsMessage::PeerGossip { key, value } => Self::PeerGossip { key, value, addr },
//             KvsMessage::PeerJoin => Self::PeerJoin { addr },
//         }
//     }
// }
