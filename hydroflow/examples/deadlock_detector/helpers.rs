use regex::Regex;
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

use rand::Rng;
pub fn decide(odds: u8) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..100) <= odds
}

pub fn parse_edge(line: String) -> Option<(u32, u32)> {
    let re = Regex::new(r"\((\d+),\s*(\d+)\)").unwrap();
    let caps = match re.captures(line.as_str()) {
        Some(c) => c,
        None => return None,
    };
    return Some((
        caps.get(1).unwrap().as_str().parse::<u32>().unwrap(),
        caps.get(2).unwrap().as_str().parse::<u32>().unwrap(),
    ));
}
