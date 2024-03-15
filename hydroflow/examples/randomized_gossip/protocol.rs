use std::collections::HashSet;
use std::net::SocketAddr;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Message {
    KnownMessages {
        messages: HashSet<ChatMessage>
    }
}

/// TODO: Document
 #[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug, Hash)]
pub struct ChatMessage {
    pub user_id: String,
    pub message: String,
    pub ts: DateTime<Utc>,
}
