use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

// Member Request
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct MemberRequest {
    pub nickname: String,
    pub connect_addr: SocketAddr,
    pub messages_addr: SocketAddr,
}
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct MemberResponse {
    pub messages_addr: SocketAddr,
}
// Member Response
//
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub nickname: String,
    pub message: String,
    pub ts: DateTime<Utc>,
}
