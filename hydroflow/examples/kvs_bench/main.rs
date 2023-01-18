mod client;
mod server;

use crate::client::run_client;
use crate::server::run_server;
use clap::Parser;
use clap::ValueEnum;
use hydroflow::tokio;
use hydroflow::util::ipv4_resolve;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum KVSRequest {
    Put { key: u64, value: u64 },
    Get { key: u64 },
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum KVSResponse {
    Response { key: u64, value: u64 },
}

#[derive(Clone, ValueEnum, Debug)]
enum Role {
    Client,
    Server,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(value_enum, long)]
    role: Role,
    #[clap(long, value_parser = ipv4_resolve)]
    addr: Option<SocketAddr>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let opts = Opts::parse();
    let addr = opts.addr.unwrap();

    match opts.role {
        Role::Client => {
            run_client(addr).await;
        }
        Role::Server => {
            run_server(addr).await;
        }
    }
}
