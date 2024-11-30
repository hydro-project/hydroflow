use regex::Regex;

use crate::protocol::ServerReq;

pub fn parse_command(line: String) -> Option<ServerReq> {
    let re = Regex::new(r"([A-z]+)\s+(.+)").unwrap();
    let caps = re.captures(line.as_str())?;

    let binding = caps.get(1).unwrap().as_str().to_uppercase();
    let cmdstr = binding.as_str();
    let args = caps.get(2).unwrap().as_str();
    match cmdstr {
        "PUT" => {
            let kv = args.split_once(',')?;
            Some(ServerReq::ClientPut {
                key: kv.0.trim().to_string(),
                value: kv.1.trim().to_string(),
            })
        }
        "GET" => Some(ServerReq::ClientGet {
            key: args.trim().to_string(),
        }),
        _ => None,
    }
}
