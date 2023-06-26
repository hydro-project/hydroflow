use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug, Hash, Copy)]
pub enum MsgType {
    VoteReq,
    Vote,
    Err,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct CoordMsg {
    pub xid: u16,
    pub mtype: MsgType,
}
/// Member Response
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct SubordResponse {
    pub xid: u16,
    pub mtype: MsgType,
}
