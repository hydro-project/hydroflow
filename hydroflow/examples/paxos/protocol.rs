use std::collections::HashSet;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Ord, PartialEq, Eq, Clone, Serialize, Deserialize, Debug, Hash, Copy)]
pub struct Ballot {
    pub id: u16,
    pub num: u16,
}

impl PartialOrd for Ballot {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.num == other.num {
            self.id.partial_cmp(&other.id)
        } else {
            self.num.partial_cmp(&other.num)
        }
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug, Hash)]
pub struct Entry {
    pub payload: String,
    pub slot: u16,
    pub ballot: Ballot,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct P1a {
    pub ballot: Ballot,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct P1b {
    pub ballot: Ballot,
    pub max_ballot: Ballot,
    pub log: HashSet<Entry>,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct P2a {
    pub entry: Entry,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug, Hash)]
pub struct P2b {
    pub entry: Entry,
    pub max_ballot: Ballot,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug, Hash)]
pub struct IAmLeader {
    pub ballot: Ballot,
}
