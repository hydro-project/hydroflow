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

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let server_addr = ipv4_resolve(opts.server_addr.clone());

    match opts.role {
        Role::Client => {
            let (outbound, inbound) = bind_udp_socket(opts.addr.clone().unwrap()).await;
            run_client(outbound, inbound, server_addr, opts.graph.clone()).await;
        }
        Role::Server => {
            let (outbound, inbound) = bind_udp_socket(opts.server_addr.clone()).await;
            run_server(outbound, inbound, opts.graph.clone()).await;
        }
    }
}
