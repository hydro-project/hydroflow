mod broadcast_receiver_stream;
mod server;
mod util;

use crate::server::run_server;
use clap::command;
use clap::Parser;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

use hydroflow::lang::lattice::crdts::EMaxReg;
use hydroflow::lang::lattice::crdts::EVClock;
use serde_big_array::BigArray;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug, Ord, PartialOrd)]
pub struct ValueType {
    #[serde(with = "BigArray")]
    pub data: [u8; 1024],
}

impl Default for ValueType {
    fn default() -> Self {
        Self { data: [0; 1024] }
    }
}

type MyRegType = EMaxReg<ValueType>;
type MyVClock = EVClock;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum KVSRequest {
    Put { key: u64, value: ValueType },
    Get { key: u64 },
    Gossip { key: u64, reg: MyRegType },
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum KVSResponse {
    PutResponse { key: u64 },
    GetResponse { key: u64, reg: MyRegType },
}

#[derive(Debug, Parser)] // requires `derive` feature
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Bench {
        #[clap(long)]
        threads: usize,

        #[clap(long)]
        dist: f64,
    },
}

fn main() {
    let ctx = tmq::Context::new();

    let throughput = Arc::new(AtomicUsize::new(0));

    match Cli::parse().command {
        Commands::Bench { threads, dist } => {
            let topology: Vec<_> = (0..threads).map(|x| x).collect();

            for addr in topology.iter() {
                run_server(
                    *addr,
                    topology.clone(),
                    dist,
                    ctx.clone(),
                    throughput.clone(),
                );
            }
        }
    }

    std::thread::sleep(Duration::from_millis(2000));

    throughput.store(0, Ordering::SeqCst);
    let start_time = std::time::Instant::now();

    std::thread::sleep(Duration::from_millis(5000));
    let puts = throughput.load(Ordering::SeqCst) as f64 / start_time.elapsed().as_secs_f64();
    println!("{puts}");

    // loop {
    //     if time_since_last_report.elapsed() >= Duration::from_secs(1) {
    //         time_since_last_report = Instant::now();
    //         println!("puts/s: {}", throughput.load(Ordering::SeqCst));
    //         throughput.store(0, Ordering::SeqCst);

    //         if start_time.elapsed() >= Duration::from_secs(5) {
    //             return;
    //         }
    //     }

    //     std::thread::sleep(Duration::from_millis(32));
    // }
}
