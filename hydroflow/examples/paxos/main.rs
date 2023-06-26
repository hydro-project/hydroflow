use std::net::SocketAddr;
use std::path::Path;

use clap::{Parser, ValueEnum};
use proposer::run_proposer;
use hydroflow::tokio;
use hydroflow::util::{ipv4_resolve};
use serde::Deserialize;
use acceptor::run_acceptor;

mod proposer;
mod helpers;
mod protocol;
mod acceptor;

#[derive(Clone, ValueEnum, Debug)]
enum Role {
    Proposer,
    Acceptor,
}

#[derive(Clone, ValueEnum, Debug)]
enum GraphType {
    Mermaid,
    Dot,
    Json,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(long)]
    path: String,
    #[clap(value_enum, long)]
    role: Role,
    #[clap(long, value_parser = ipv4_resolve)]
    addr: SocketAddr,
    #[clap(value_enum, long)]
    graph: Option<GraphType>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    proposers: Vec<String>,
    acceptors: Vec<String>,
    f: u16,
    i_am_leader_resend_timeout: u16,
    i_am_leader_check_timeout_node_0: u16,
    i_am_leader_check_timeout_other_nodes: u16,
}

#[hydroflow::main]
async fn main() {
    let opts = Opts::parse();
    let path = Path::new(&opts.path);
    let addr = opts.addr;

    match opts.role {
        Role::Proposer => {
            run_proposer(addr, path, opts.graph.clone()).await;
        }
        Role::Acceptor => {
            run_acceptor(addr, opts.graph.clone()).await;
        }
    }
}
