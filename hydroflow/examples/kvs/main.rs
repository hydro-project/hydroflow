#![feature(never_type)]

extern crate hdrhistogram;

use std::time::{Duration, Instant};

use clap::Parser;
use common::Dist;

use crate::kvs_compiled::WorkerConfig;

mod common;
mod kvs_compiled;

#[derive(Clone, Parser)]
struct Args {
    /// What distribution to use. Either `uniform,n` or `zipf,n,theta`
    #[clap(long)]
    dist: Dist,
    /// How long to run the benchmark for, in seconds
    #[clap(long)]
    run_seconds: u64,
    /// How many workers to run with
    #[clap(long)]
    workers: u64,
    /// The size of each batch generated by the workload operator
    #[clap(long)]
    workload_batch_size: usize,
    /// How long to wait before flushing data out to other workers, in milliseconds
    #[clap(long)]
    epoch_millis: u64,
}

fn main() {
    let args = Args::parse();

    // let kvs = args.implementation;
    let dist = args.dist.clone();
    let epoch = Duration::from_millis(args.epoch_millis);

    let config = WorkerConfig {
        epoch,
        dist,
        batch_size: args.workload_batch_size,
    };

    let kvs = kvs_compiled::Kvs::<u64, u64>::new(args.workers, config);
    let run_duration = Duration::from_millis(args.run_seconds * 1000);

    let start = Instant::now();

    let mut data: Vec<_> = (0..args.workers).map(|_| 0).collect();
    while start.elapsed() < run_duration {
        if let Ok(Some(v)) = (*kvs.monitor).borrow_mut().try_next() {
            data[v.from] = v.data;
        }
    }

    let sum: usize = data.iter().sum();
    println!("{},{}", args.workers, sum);
}
