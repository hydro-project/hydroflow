#![feature(addr_parse_ascii)]
use clap::{ArgEnum, Parser};
use client::run_client;
use hydroflow::tokio;
use server::run_server;
use std::net::{SocketAddr, ToSocketAddrs};

mod client;
mod helpers;
mod protocol;
mod server;

#[derive(Clone, ArgEnum, Debug)]
enum Role {
    Client,
    Server,
}
#[derive(Clone, ArgEnum, Debug)]
enum GraphType {
    Mermaid,
    Dot,
    Json,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(arg_enum, long)]
    role: Role,
    #[clap(long)]
    addr: Option<String>,
    #[clap(long)]
    server_addr: String,
    #[clap(arg_enum, long)]
    graph: Option<GraphType>,
}

pub fn resolve_ipv4_connection_addr(server_addr: String) -> Option<SocketAddr> {
    let mut addrs = server_addr.to_socket_addrs().unwrap();

    addrs.find(|addr| addr.is_ipv4())
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    // get server_addr
    println!("Server address: {}", opts.server_addr);
    let server_addr = resolve_ipv4_connection_addr(opts.server_addr.clone()).unwrap_or_else(|| {
        println!("Error parsing server address");
        panic!();
    });
    match opts.role {
        Role::Client => {
            let client_addr = resolve_ipv4_connection_addr(opts.addr.clone().unwrap())
                .unwrap_or_else(|| {
                    println!("Error parsing server address");
                    panic!();
                });
            run_client(opts, server_addr, client_addr).await;
        }
        Role::Server => {
            run_server(opts, server_addr).await;
        }
    }
}
