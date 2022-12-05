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
