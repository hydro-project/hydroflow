use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Timestamped<T: Debug> {
    pub timestamp: isize,
    pub data: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Payload<T: Debug> {
    pub key: u64,
    pub contents: Timestamped<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OperationPayload {
    pub key: u64,
    pub change: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct QueryResponse {
    pub key: u64,
    pub value: i64,
}
