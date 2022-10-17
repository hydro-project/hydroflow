use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Message {
    ConnectRequest {
        nickname: String,
        addr: SocketAddr,
    },

    ConnectResponse,

    ChatMsg {
        nickname: String,
        message: String,
        ts: DateTime<Utc>,
    },
}
