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
    pub addr: String,
    pub slot: u16,
    pub ballot: u16,
    pub pid: u16,
    pub val: i32,
    pub mtype: MsgType,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct ClientReq {
    pub val: i32,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct AcceptorResponse {
    pub slot: u16,
    pub ballot: u16,
    pub pid: u16,
    pub accepted_val: Option<i32>,
    pub accepted_ballot: Option<u16>,
    // pub accepted_pid: u16,
    pub win: bool, // whether the acceptor has chosen the given proposer as a leader
    pub val: Option<i32>,
    pub mtype: MsgType,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum Msg {
    AcceptorRes(AcceptorResponse),
    ClientReq(ClientReq),
}
