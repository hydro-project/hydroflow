use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct Msg {
    pub val: u64,
    pub idx: u64,
    pub ts: DateTime<Utc>,
    pub nonce: Vec<u8>,
}
