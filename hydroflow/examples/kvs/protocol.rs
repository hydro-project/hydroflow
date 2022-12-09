use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum KVSMessage {
    Put { key: String, value: String },
    Get { key: String },
    Response { key: String, value: String },
}
