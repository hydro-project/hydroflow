use serde::{Deserialize, Serialize};

/// Message pattern:
/// User input on stdin
/// Phase 1
///  Coordinator sends Prepare
///  Subordinate responds with Commit or Abort
///  Coordinator collects, wait for all responses, logs Commit if all commit
/// Phase 2
///  Coordinator sends Commit after flushing log, or just sends Abort
///  Subordinate responds with AckP2
///  Coordinator collects, wait for all responses, sends End
///  Subordinate responds with Ended
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
    pub mtype: MsgType,
}
/// Member Response
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct SubordResponse {
    pub xid: u16,
    pub mtype: MsgType,
}
