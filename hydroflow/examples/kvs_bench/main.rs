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
    Response { key: u64, reg: MyMVReg },
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
        #[clap(long, value_parser = ipv4_resolve)]
        addr: SocketAddr,
    },
    #[command(arg_required_else_help = true)]
    Server {
        #[clap(long, value_parser = ipv4_resolve)]
        addr: SocketAddr,

        #[clap(long, value_parser = ipv4_resolve, value_delimiter = ',')]
        peers: Vec<SocketAddr>,
    },
}

pub fn ipv4_resolve_multi(addr: &str) -> Result<Vec<SocketAddr>, std::io::Error> {
    println!("{addr}");
    let vec = addr.split(',').map(|x| ipv4_resolve(x).unwrap()).collect();

    println!("{vec:?}");

    Ok(vec)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    match Cli::parse().command {
        Commands::Client { addr } => run_client(addr).await,
        Commands::Server { addr, mut peers } => {
            peers.retain(|&x| x != addr); // Don't try to connect to self, makes the bash script easier to write.
            run_server(addr, peers).await
        }
    }
}
