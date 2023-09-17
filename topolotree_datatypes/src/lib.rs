use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Payload<T: Debug> {
    pub timestamp: isize,
    pub data: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OperationPayload {
    pub change: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IncrementRequest {
    tweet_id: u64,
    likes: i32,
}
