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
use hydroflow::tokio;
use hydroflow::util::ipv4_resolve;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

type MyMVReg = MVReg<u64, SocketAddr>;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum KVSRequest {
    Put { key: u64, value: u64 },
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
        #[clap(long, value_parser = ipv4_resolve, value_delimiter = ',')]
        targets: Vec<SocketAddr>,
    },
    #[command(arg_required_else_help = true)]
    Server {
        #[clap(long, value_parser = ipv4_resolve)]
        addr: SocketAddr,

        #[clap(long, value_parser = ipv4_resolve, value_delimiter = ',')]
        topology: Vec<SocketAddr>,
    },
}

#[tokio::main(flavor = "current_thread")]
// #[tokio::main]
async fn main() {
    // run_server("127.0.0.1:5000".parse().unwrap(), vec![]).await;

    match Cli::parse().command {
        Commands::Client { targets } => run_client(targets).await,
        Commands::Server { addr, mut topology } => {
            topology.retain(|&x| x != addr); // Don't try to connect to self, makes the bash script easier to write.
            let peers = topology;
            run_server(addr, peers).await
        }
    }
}
