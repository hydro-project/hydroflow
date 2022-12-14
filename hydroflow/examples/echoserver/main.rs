use clap::{ArgEnum, Parser};
use client::run_client;
use hydroflow::tokio;
use hydroflow::util::{bind_udp_socket, ipv4_resolve};
use server::run_server;

mod client;
mod protocol;
mod server;

#[derive(Clone, ArgEnum, Debug)]
enum Role {
    Client,
    Server,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(arg_enum, long)]
    role: Role,
    #[clap(long)]
    client_addr: Option<String>,
    #[clap(long)]
    server_addr: String,
}

#[tokio::main]
async fn main() {
    // parse command line arguments
    let opts = Opts::parse();

    // depending on the role, pass in arguments to the right function
    match opts.role {
        Role::Server => {
            // allocate `outbound` and `inbound` sockets
            let (outbound, inbound) = bind_udp_socket(opts.server_addr.clone()).await;
            // run the server
            run_server(outbound, inbound).await;
        }
        Role::Client => {
            // resolve the server's IP address
            let server_addr = ipv4_resolve(opts.server_addr.clone());
            // allocate `outbound` and `inbound` sockets
            let (outbound, inbound) = bind_udp_socket(opts.client_addr.clone().unwrap()).await;
            // run the client
            run_client(outbound, inbound, server_addr).await;
        }
    }
}
