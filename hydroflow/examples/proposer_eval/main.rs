use clap::{ArgEnum, Parser};
// use coordinator::run_coordinator;
use acceptor_blank::run_acceptor;
use hydroflow::tokio;
use proposer::run_proposer;
use proxy_leader::run_proxy_leader;
use serde::Deserialize;

use core::future::Future;
use futures::executor::block_on;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::ops::Index;
use std::path::Path;
use std::thread;
use tokio::task;

mod acceptor_blank;
mod proposer;
mod protocol;
mod proxy_leader;

struct Addresses {
    coordinator: String,
    subordinates: Vec<String>,
}

// fn read_addresses_from_file<P: AsRef<Path>>(path: P) -> Result<Addresses, Box<dyn Error>> {
//     // Open the file in read-only mode with buffer.
//     let file = File::open(path)?;
//     let reader = BufReader::new(file);

//     // Read the JSON contents of the file as an instance of `addresses`.
//     let u = serde_json::from_reader(reader)?;

//     // Return the addresses.
//     Ok(u)
// }

#[derive(Clone, ArgEnum, Debug)]
enum Role {
    Proposer,
    Acceptor,
    ProxyLeader,
}


#[derive(Parser, Debug)]
struct CLIOpts {
    #[clap(arg_enum, long)]
    role: Role,
    #[clap(long)]
    port: u16,
    #[clap(long)]
    addr: String,
    #[clap(long)]
    id: u16,
    #[clap(long)]
    use_proxy: bool,
    #[clap(long, default_value_t=3)]
    acceptors: u32,
    // #[clap(long)]
    // output_dir: String,
}

struct Opts {
    // #[clap(long)]
    // path: String,
    // contain CLIOpts instance
    port: u16,
    addr: String,
    id: u16,
    use_proxy: bool,
    acceptor_addrs: Vec<String>,
    proxy_addrs: Vec<String>,
}

#[tokio::main]
async fn main() {
    let mut cli_opts = CLIOpts::parse();
    // create new Opts
    let mut opts = Opts {
        // path: cli_opts.path,
        port: cli_opts.port,
        addr: cli_opts.addr,
        id: cli_opts.id,
        use_proxy: cli_opts.use_proxy,
        acceptor_addrs: vec![],
        proxy_addrs: vec![],
        // output_dir: cli_opts.output_dir,
    };

    for port in 1400..1400+cli_opts.acceptors {
        opts.acceptor_addrs.push(String::from(format!("localhost:{}", port)));
    }

    opts.proxy_addrs.push(String::from("localhost:1200"));
    opts.proxy_addrs.push(String::from("localhost:1201"));
    opts.proxy_addrs.push(String::from("localhost:1202"));

    // opts.use_proxy = false;
    // println!("{:?}", opts.use_proxy);

    match cli_opts.role {
        Role::Proposer => run_proposer(opts).await,
        Role::Acceptor => run_acceptor(opts.port).await,
        Role::ProxyLeader => run_proxy_leader(opts).await,
    }
}
