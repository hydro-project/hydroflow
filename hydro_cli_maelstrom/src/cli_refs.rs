use std::collections::HashMap;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerPort {
    TcpPort(SocketAddr),
    Demux(HashMap<u32, ServerPort>),
    Merge(Vec<ServerPort>),
    Tagged(Box<ServerPort>, u32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerBindConfig {
    TcpPort(String),
    Merge(Vec<ServerBindConfig>),
    Tagged(Box<ServerBindConfig>, u32),
}
