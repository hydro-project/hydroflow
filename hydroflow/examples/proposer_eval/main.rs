use clap::{ArgEnum, Parser};
// use coordinator::run_coordinator;
use acceptor_blank::run_acceptor;
use hydroflow::tokio;
use proposer::run_proposer;
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
}

#[derive(Parser, Debug)]
struct Opts {
    // #[clap(long)]
    // path: String,
    // #[clap(arg_enum, long)]
    // role: Role,
    #[clap(long)]
    port: u16,
    #[clap(long)]
    addr: String,
    #[clap(long)]
    id: u16,

    acceptor_addrs: Vec<String>,
}

#[tokio::main]
async fn main() {
    let mut opts = Opts::parse();
    opts.acceptor_addrs.push(String::from("localhost:1400"));
    opts.acceptor_addrs.push(String::from("localhost:1401"));
    opts.acceptor_addrs.push(String::from("localhost:1402"));
    // let path = Path::new(&opts.path);
    // let subordinates = read_addresses_from_file(path).unwrap().subordinates;
    // let coordinator = read_addresses_from_file(path).unwrap().coordinator;
    // run_acceptor(opts).await;f

    for addr in opts.acceptor_addrs.iter() {
        let port = addr.split(":").last().unwrap().parse::<u16>().unwrap();
        thread::spawn(move || {
            block_on(run_acceptor(port));
        });
    }

    run_proposer(opts).await;
}
