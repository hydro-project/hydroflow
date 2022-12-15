use clap::{Parser, ValueEnum};
use coordinator::run_coordinator;
use hydroflow::tokio;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve};
use serde::Deserialize;
use subordinate::run_subordinate;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::Path;

mod coordinator;
mod helpers;
mod protocol;
mod subordinate;

/// This is a remedial 2PC implementation.

#[derive(Clone, ValueEnum, Debug)]
enum Role {
    Coordinator,
    Subordinate,
}

#[derive(Clone, ValueEnum, Debug)]
enum GraphType {
    Mermaid,
    Dot,
    Json,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(long)]
    path: String,
    #[clap(value_enum, long)]
    role: Role,
    #[clap(long, value_parser = ipv4_resolve)]
    addr: SocketAddr,
    #[clap(value_enum, long)]
    graph: Option<GraphType>,
}

#[derive(Deserialize, Debug)]
struct Addresses {
    coordinator: String,
    subordinates: Vec<String>,
}

fn read_addresses_from_file<P: AsRef<Path>>(path: P) -> Result<Addresses, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Subordinates`.
    let u = serde_json::from_reader(reader)?;

    // Return the `Subordinates`.
    Ok(u)
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let path = Path::new(&opts.path);
    let subordinates = read_addresses_from_file(path).unwrap().subordinates;
    let coordinator = read_addresses_from_file(path).unwrap().coordinator;
    let addr = opts.addr;

    match opts.role {
        Role::Coordinator => {
            let (outbound, inbound, _) = bind_udp_bytes(addr).await;
            run_coordinator(outbound, inbound, subordinates, opts.graph.clone()).await;
        }
        Role::Subordinate => {
            let (outbound, inbound, _) = bind_udp_bytes(addr).await;
            println!("Coordinator: {}", coordinator);
            let server_addr = ipv4_resolve(coordinator.trim()).unwrap();

            run_subordinate(outbound, inbound, server_addr, opts.graph.clone()).await;
        }
    }
}
