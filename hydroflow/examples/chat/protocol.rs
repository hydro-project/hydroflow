use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Message {
    ConnectRequest,
    ConnectResponse,
    ChatMsg {
        nickname: String,
        message: String,
        ts: DateTime<Utc>,
    },
}
