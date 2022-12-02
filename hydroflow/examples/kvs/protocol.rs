use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum KVSMessage {
    Put {
        client: SocketAddr,
        key: String,
        value: String,
    },
    Get {
        client: SocketAddr,
        key: String,
    },
    Response {
        key: String,
        value: String,
    },
}
