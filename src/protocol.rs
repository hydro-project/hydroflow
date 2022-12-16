use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum Message {
    Echo { payload: String, ts: DateTime<Utc> },
    Heartbeat,
    HeartbeatAck,
}
