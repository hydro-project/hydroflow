use crate::protocol::KVSMessage;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio_util::codec::LinesCodecError;

pub fn serialize_msg<T>(msg: T) -> String
where
    T: Serialize + for<'a> Deserialize<'a> + Clone,
{
    let r = json!(msg);
    let s = r.to_string();
    let t = s;
    t
}

pub fn deserialize_msg<T>(msg: Result<(String, SocketAddr), LinesCodecError>) -> T
where
    T: Serialize + for<'a> Deserialize<'a> + Clone,
{
    let body = msg.unwrap().0;
    let res = serde_json::from_str(&body);
    res.unwrap()
}

pub fn is_put(m: KVSMessage) -> Option<KVSMessage> {
    match m {
        KVSMessage::Put {
            client: _,
            key: _,
            value: _,
        } => Some(m),
        _ => None,
    }
}

pub fn is_get(m: KVSMessage) -> Option<KVSMessage> {
    match m {
        KVSMessage::Get { client: _, key: _ } => Some(m),
        _ => None,
    }
}

pub fn is_resp(m: KVSMessage) -> Option<KVSMessage> {
    match m {
        KVSMessage::Response { key: _, value: _ } => Some(m),
        _ => None,
    }
}

pub fn resolve_ipv4_connection_addr(server_ip: String, server_port: u16) -> Option<SocketAddr> {
    let mut addrs = format!("{}:{}", server_ip, server_port)
        .to_socket_addrs()
        .unwrap();

    addrs.find(|addr| addr.is_ipv4())
}

pub fn parse_command(line: String, client: SocketAddr) -> Option<KVSMessage> {
    let re = Regex::new(r"([A-z]+)\s+(.+)").unwrap();
    let caps = re.captures(line.as_str())?;

    let binding = caps.get(1).unwrap().as_str().to_uppercase();
    let cmdstr = binding.as_str();
    let args = caps.get(2).unwrap().as_str();
    let cmd = match cmdstr {
        "PUT" => {
            let kv = args.split_once(',')?;
            Some(KVSMessage::Put {
                client: client,
                key: kv.0.trim().to_string(),
                value: kv.1.trim().to_string(),
            })
        }
        "GET" => Some(KVSMessage::Get {
            client: client,
            key: args.trim().to_string(),
        }),
        _ => None,
    }?;

    Some(cmd)
}
