use chrono::prelude::*;
use serde::{Deserialize, Serialize};

// Member Request
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct MemberRequest {
    pub nickname: String,
    pub connect_addr: String,
    pub messages_addr: String,
}
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct MemberResponse {
    pub messages_port: u16,
}
// Member Response
//
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub nickname: String,
    pub message: String,
    pub ts: DateTime<Utc>,
}
