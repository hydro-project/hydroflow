use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_util::codec::LinesCodecError;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug, Hash, Copy)]
pub enum MsgType {
    Prepare,
    Commit,
    Abort,
    AckP2,
    End,
    Ended,
    Err,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct CoordMsg {
    pub xid: u16,
    pub mid: u16,
    pub mtype: MsgType,
}
/// Member Response
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct SubordResponse {
    pub xid: u16,
    pub mid: u16,
    pub addr: String,
    pub mtype: MsgType,
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
