use clap::{ArgEnum, Parser};
use client::run_client;
use hydroflow::tokio;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve};
use server::run_server;

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
    addr: String,
    #[clap(long)]
    server_addr: Option<String>,
    #[clap(arg_enum, long)]
    graph: Option<GraphType>,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let addr = ipv4_resolve(opts.addr.clone());

    match opts.role {
        Role::Client => {
            let (outbound, inbound) = bind_udp_bytes(addr).await;
            println!("Client is bound to {}", opts.addr.clone());
            println!(
                "Attempting to connect to server at {}",
                opts.server_addr.clone().unwrap()
            );
            let server_addr = ipv4_resolve(opts.server_addr.clone().unwrap());
            run_client(outbound, inbound, server_addr, opts.graph.clone()).await;
        }
        Role::Server => {
            let (outbound, inbound) = bind_udp_bytes(addr).await;
            println!("Listening on {}", opts.addr.clone());
            run_server(outbound, inbound, opts.graph.clone()).await;
        }
    }
}
