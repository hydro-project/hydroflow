use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug, Hash, Copy)]
pub enum MsgType {
    P1A,
    P1B,
    P2A,
    P2B,
}

// Proposer
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct ProposerMsg {
    pub slot: u16,
    pub ballot: u16,
    pub pid: u16,
    pub val: u16,
    pub mtype: MsgType,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct AcceptorResponse {
    pub slot: u16,
    pub ballot: u16,
    pub pid: u16,
    pub accepted_val: u16, 
    pub accepted_ballot: u16,
    // pub accepted_pid: u16,
    pub val: u16,
    pub mtype: MsgType,
}