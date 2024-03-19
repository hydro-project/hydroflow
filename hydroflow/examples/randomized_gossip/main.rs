use std::collections::HashSet;
use std::net::SocketAddr;

use chrono::Utc;
use clap::{Args, Parser, Subcommand, ValueEnum};
use itertools::Itertools;

use hydroflow::hydroflow_syntax;
use hydroflow::util::ipv4_resolve;
use hydroflow_lang::graph::{WriteConfig, WriteGraphType};

use crate::protocol::ChatMessage;
use crate::server::run_server;

mod protocol;
mod client;
mod server;

#[derive(Clone, ValueEnum, Debug, Eq, PartialEq)]
enum Role {
    Server1,
    Server2,
    Server3,
    Server4,
    Server5,
    Client,
}

impl Role {
    fn listening_address(&self) -> SocketAddr {
        match self {
            // TODO: Cleanup consts.
            Role::Server1 => ipv4_resolve("localhost:54321"),
            Role::Server2 => ipv4_resolve("localhost:54322"),
            Role::Server3 => ipv4_resolve("localhost:54323"),
            Role::Server4 => ipv4_resolve("localhost:54324"),
            Role::Server5 => ipv4_resolve("localhost:54325"),
            Role::Client => ipv4_resolve("localhost:0"), // Let the OS assign
        }
        .unwrap()
    }
}

#[derive(Parser, Debug)]
struct Opts {
    #[command(subcommand)]
    command: Commands,
    #[clap(long)]
    graph: Option<WriteGraphType>,
    #[clap(flatten)]
    write_config: Option<WriteConfig>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Runs an instance of a client
    Client {
        /// The nickname / user_id for the connected client.
        #[arg(long)]
        name: String,
        /// The server to connect to.
        #[arg(long)]
        server: Role
    },
    /// Runs an instance of a server
    Server {
        /// One of the several server roles available.
        #[arg(long)]
        role: Role
    }
}


#[hydroflow::main]
async fn main() {
    let opts = Opts::parse();
    match opts.command {
        Commands::Client { .. }  => {
            todo!("Pending Impl")
        }
        Commands::Server { .. } => {
                run_server(opts).await;
        }

    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}
