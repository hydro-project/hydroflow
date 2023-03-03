mod broadcast_receiver_stream;
mod client;
mod server;
mod util;

use crate::client::run_client;
use crate::server::run_server;
use clap::command;
use clap::Parser;
use clap::Subcommand;
use crdts::MVReg;
use crdts::VClock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::time::Duration;

use serde_big_array::BigArray;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug, Ord, PartialOrd)]
pub struct ValueType {
    #[serde(with = "BigArray")]
    pub data: [u8; 1024],
}

type MyMVReg = MVReg<ValueType, String>;
type MyVClock = VClock<String>;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum KVSRequest {
    Put { key: u64, value: ValueType },
    Get { key: u64 },
    Gossip { key: u64, reg: MyMVReg },
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum KVSResponse {
    PutResponse { key: u64 },
    GetResponse { key: u64, reg: MyMVReg },
}

#[derive(Debug, Parser)] // requires `derive` feature
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Client {
        #[clap(long, value_delimiter = ',')]
        targets: Vec<String>,
    },
    #[command(arg_required_else_help = true)]
    Servers {
        #[clap(long, value_delimiter = ',')]
        servers: Vec<String>,

        #[clap(long, value_delimiter = ',')]
        gossip: Vec<String>,
    },
    // All {
    //     #[clap(long, value_delimiter = ',')]
    //     clients: Vec<String>,

    //     #[clap(long, value_delimiter = ',')]
    //     servers: Vec<String>,
    // },
}

fn main() {
    // run_server("127.0.0.1:5000".parse().unwrap(), vec![]).await;

    let args: Vec<_> = env::args().collect();
    println!("{:?}", args);

    let ctx = tmq::Context::new();

    match Cli::parse().command {
        Commands::Client { targets } => run_client(targets, ctx),
        Commands::Servers { servers, gossip } => {
            assert_eq!(servers.len(), gossip.len());

            for i in 0..servers.len() {
                run_server(
                    servers[i].clone(),
                    gossip[i].clone(),
                    gossip.clone(),
                    ctx.clone(),
                );
            }
        } // Commands::All { clients, servers } => {
          //     let mut topology = HashMap::new();
          //     for idx in 0..servers.len() {
          //         topology.insert(idx as u64, servers[idx].clone());
          //     }

          //     for server in servers {
          //         run_server(server, topology.clone(), ctx.clone());
          //     }

          //     std::thread::sleep(Duration::from_secs(1));

          //     run_client(clients, ctx);
          // }
    }

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
