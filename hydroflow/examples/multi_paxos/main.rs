use clap::{ArgEnum, Parser};
// use coordinator::run_coordinator;
use hydroflow::tokio;
use serde::Deserialize;
// use subordinate::run_subordinate;
use acceptor::run_acceptor;
use proposer::run_proposer;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

mod acceptor;
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
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    // let path = Path::new(&opts.path);
    // let subordinates = read_addresses_from_file(path).unwrap().subordinates;
    // let coordinator = read_addresses_from_file(path).unwrap().coordinator;
    // run_acceptor(opts).await;
    run_proposer(opts).await;
}
