use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tokio_util::codec::LinesCodecError;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Message {
    ConnectRequest {
        nickname: String,
        addr: SocketAddr,
    },

    ConnectResponse,

    ChatMessage {
        nickname: String,
        message: String,
        ts: DateTime<Utc>,
    },
}

pub fn serialize_msg<T>(msg: T) -> String
where
    T: Serialize + for<'a> Deserialize<'a> + Clone,
{
    json!(msg).to_string()
}

pub fn deserialize_msg<T>(msg: Result<(String, SocketAddr), LinesCodecError>) -> T
where
    T: Serialize + for<'a> Deserialize<'a> + Clone,
{
    serde_json::from_str(&(msg.unwrap().0)).unwrap()
}
