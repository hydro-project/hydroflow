use clap::{ArgEnum, Parser};
use client::run_client;
use hydroflow::tokio;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve};
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
    #[clap(long)]
    name: String,
    #[clap(arg_enum, long)]
    role: Role,
    #[clap(long)]
    client_addr: Option<String>,
    #[clap(long)]
    server_addr: String,
    #[clap(arg_enum, long)]
    graph: Option<GraphType>,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let server_str = opts.server_addr.clone();

    match opts.role {
        Role::Client => {
            let client_str = opts.client_addr.clone().unwrap();
            println!(
                "Client is bound to {}, connecting to Server at {}",
                client_str.clone(),
                server_str.clone()
            );
            let client_addr = ipv4_resolve(client_str);
            let (outbound, inbound) = bind_udp_bytes(client_addr).await;
            run_client(
                outbound,
                inbound,
                ipv4_resolve(server_str.clone()),
                opts.name.clone(),
                opts.graph.clone(),
            )
            .await;
        }
        Role::Server => {
            println!("Listening on {}", server_str.clone());
            let server_addr = ipv4_resolve(server_str);
            let (outbound, inbound) = bind_udp_bytes(server_addr).await;

            run_server(outbound, inbound, opts.graph.clone()).await;
        }
    }
}
