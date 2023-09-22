use std::error::Error;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::utils;

/// A message from maelstrom
#[derive(Serialize, Deserialize)]
pub struct Message<T> {
    pub src: String,
    pub dest: String,
    pub body: T,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename = "init")]
pub struct InitMsg {
    pub msg_id: i32,
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename = "init_ok")]
pub struct InitOkMsg {
    pub in_reply_to: i32,
}

impl Message<InitMsg> {
    pub fn response(&self) -> Result<Message<InitOkMsg>, Box<dyn Error>> {
        let body = self.body.response();
        Ok(Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body,
        })
    }
}

impl InitMsg {
    fn response(&self) -> InitOkMsg {
        InitOkMsg {
            in_reply_to: self.msg_id,
        }
    }
}

/// Configuration information returned by maelstrom's init package.
pub struct MaelstromConfig {
    /// The node id of this node.
    pub node_id: usize,
    /// Node ids are determined by the index of the corresponding name in the node_names vector.
    pub node_names: Vec<String>,
    pub node_count: usize,
}

impl TryFrom<Message<InitMsg>> for MaelstromConfig {
    type Error = &'static str;

    fn try_from(msg: Message<InitMsg>) -> Result<Self, Self::Error> {
        let node_names = msg.body.node_ids;
        // Find the node_id (index in node_names) which corresponds to the current node's name
        let node_id = node_names
            .iter()
            .position(|x| *x == msg.body.node_id)
            .ok_or("Could not find current node in node list")?;

        let node_count = node_names.len();

        Ok(Self {
            node_id,
            node_names,
            node_count,
        })
    }
}

/// Recieve & Ack the init payload from Maelstrom.
pub async fn maelstrom_init() -> Result<MaelstromConfig, Box<dyn Error>> {
    // Read the init message from Maelstrom
    let line = utils::read_line().await?;
    let init_msg: Message<InitMsg> = serde_json::from_str(&line)?;

    // Send an init_ok back to Maelstrom
    let response = init_msg.response()?;
    let response = serde_json::to_string(&response)?;
    println!("{}", response);

    // Extract the contents from the init message
    let cfg = init_msg.try_into()?;
    Ok(cfg)
}

/// An unknown message body
#[derive(Serialize, Deserialize)]
pub struct UnknownBody {
    #[serde(rename = "type")]
    pub maelstrom_type: String,
}

/// The body of a custom message
#[derive(Serialize, Deserialize)]
pub struct CustomBody {
    #[serde(rename = "type")]
    pub maelstrom_type: String,
    pub text: String,
}

/// The body of a maelstrom message sent from hydro
#[derive(Serialize, Deserialize)]
pub struct HydroBody {
    /// in_reply_to should be a pair of (origin node name, original message id) because that is how msg_ids from maelstrom are wrapped
    pub in_reply_to: (String, Value),
}
