use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct CoordMsg {
    pub payload: String,
}
/// Member Response
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct SubordResponse {
    pub payload: String,
}
