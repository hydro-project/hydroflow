use clap::{ArgEnum, Parser};
use client::run_client;
use hydroflow::tokio;
use server::run_server;

mod client;
mod protocol;
mod server;

// This is a distributed version of the covid tracing app. It somewhat
// arbitrarily splits apart two "responsibilities" of the app to exercise
// network communication.

#[derive(Clone, ArgEnum, Debug)]
enum Role {
    Client,
    Server,
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
    mermaid: bool,
    #[clap(long)]
    dot: bool,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    match opts.role {
        Role::Client => {
            run_client(opts).await;
        }
        Role::Server => {
            run_server(opts).await;
        }
    }
}
