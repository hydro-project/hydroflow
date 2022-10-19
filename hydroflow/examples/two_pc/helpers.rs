use crate::protocol::{CoordMsg, MsgType, SubordResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tokio_util::codec::LinesCodecError;

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

pub fn is_prepare_msg(m: CoordMsg) -> Option<CoordMsg> {
    match m.mtype {
        MsgType::Prepare {} => Some(m),
        _ => None,
    }
}

pub fn is_end_msg(m: CoordMsg) -> Option<CoordMsg> {
    match m.mtype {
        MsgType::End {} => Some(m),
        _ => None,
    }
}

pub fn is_coord_p2(m: CoordMsg) -> Option<CoordMsg> {
    match m.mtype {
        MsgType::Abort => Some(m),
        MsgType::Commit => Some(m),
        _ => None,
    }
}

// No-Op in current implementation
// pub fn is_ended_msg(m: SubordResponse) -> Option<SubordResponse> {
//     match m.mtype {
//         MsgType::Ended {} => Some(m),
//         _ => None,
//     }
// }

pub fn is_vote(m: SubordResponse) -> Option<SubordResponse> {
    match m.mtype {
        MsgType::Commit => Some(m),
        MsgType::Abort => Some(m),
        _ => None,
    }
}

pub fn is_subord_abort(m: SubordResponse) -> Option<SubordResponse> {
    match m.mtype {
        MsgType::Abort => Some(m),
        _ => None,
    }
}

pub fn is_subord_commit(m: SubordResponse) -> Option<SubordResponse> {
    match m.mtype {
        MsgType::Commit => Some(m),
        _ => None,
    }
}

pub fn is_ackp2_msg(m: SubordResponse) -> Option<SubordResponse> {
    match m.mtype {
        MsgType::AckP2 {} => Some(m),
        _ => None,
    }
}

pub fn parse_out<T: std::str::FromStr>(line: String) -> Option<T> {
    match line.trim().parse::<T>() {
        Ok(the_xid) => Some(the_xid),
        Err(_) => None,
    }
}

use rand::Rng;
pub fn decide(odds: u8) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..100) <= odds
}
