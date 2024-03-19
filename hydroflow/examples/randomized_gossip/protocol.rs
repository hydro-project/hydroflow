use std::collections::HashSet;
use std::net::SocketAddr;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum ProtocolMessage {
    ChatMessage { msg: ChatMessage },
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug, Hash)]
pub struct ChatMessage {
    pub nickname: String,
    pub message: String,
    pub ts: DateTime<Utc>,
}
