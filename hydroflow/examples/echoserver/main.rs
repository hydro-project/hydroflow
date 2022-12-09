#![feature(fmt_internals, print_internals)]
use clap::{ArgEnum, Parser};
use client::run_client;
use hydroflow::tokio;
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
    addr: Option<String>,
    #[clap(long)]
    server_addr: String,
    #[clap(arg_enum, long)]
    graph: Option<GraphType>,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    // get server_addr
    println!("Server address: {}", opts.server_addr);
    match opts.role {
        Role::Client => {
            run_client(opts).await;
        }
        Role::Server => {
            run_server(opts).await;
        }
    }
}
