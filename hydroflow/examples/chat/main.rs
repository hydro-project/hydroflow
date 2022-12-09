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
    #[clap(long)]
    name: String,
    #[clap(arg_enum, long)]
    role: Role,
    #[clap(long)]
    port: u16,
    #[clap(long)]
    addr: String,
    #[clap(long)]
    server_addr: Option<String>,
    #[clap(long)]
    server_port: Option<u16>,
    #[clap(arg_enum, long)]
    graph: Option<GraphType>,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    match opts.role {
        Role::Client => {
            let client_str = format!("{}:{}", opts.addr.clone(), opts.port.clone());
            let server_str = format!(
                "{}:{}",
                opts.server_addr.clone().unwrap(),
                opts.server_port.clone().unwrap()
            );
            println!(
                "Client is bound to {}, connecting to Server at {}",
                client_str.clone(),
                server_str.clone()
            );
            let (outbound, inbound) = bind_udp_socket(client_str.clone()).await;
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
            let server_str = format!("{}:{}", opts.addr.clone(), opts.port.clone());
            let (outbound, inbound) = bind_udp_socket(server_str.clone()).await;
            println!("Listening on {}", server_str);

            run_server(outbound, inbound, opts.graph.clone()).await;
        }
    }
}
