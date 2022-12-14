use clap::{Parser, ValueEnum};
use client::run_client;
use hydroflow::tokio;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve};
use server::run_server;
use std::net::SocketAddr;

mod client;
mod protocol;
mod server;

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
    client_addr: Option<SocketAddr>,
    #[clap(long, value_parser = ipv4_resolve)]
    server_addr: Option<SocketAddr>,
}

#[tokio::main]
async fn main() {
    // parse command line arguments
    let opts = Opts::parse();
    let server_addr = opts.server_addr.unwrap();

    // depending on the role, pass in arguments to the right function
    match opts.role {
        Role::Server => {
            // allocate `outbound` and `inbound` sockets
            let (outbound, inbound) = bind_udp_bytes(server_addr).await;
            run_server(outbound, inbound).await;
        }
        Role::Client => {
            // resolve the server's IP address
            let client_addr = opts.client_addr.unwrap();
            // allocate `outbound` and `inbound` sockets
            let (outbound, inbound) = bind_udp_bytes(client_addr).await;
            // run the client
            run_client(outbound, inbound, server_addr).await;
        }
    }
}
