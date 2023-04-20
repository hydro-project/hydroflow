mod buffer_pool;
mod protocol;
mod server;
mod util;

use crate::server::run_server;

use clap::command;
use clap::Parser;
use clap::Subcommand;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Bench {
        #[clap(long, default_value_t = 1)]
        threads: usize,

        #[clap(long, default_value_t = 4.0)]
        dist: f64,

        #[clap(long, default_value_t = 2)]
        warmup: u64,

        #[clap(long, default_value_t = 10)]
        duration: u64,

        #[clap(long, default_value_t = false)]
        report: bool,
    },
}

fn main() {
    let ctx = tmq::Context::new();

    let throughput = Arc::new(AtomicUsize::new(0));

    match Cli::parse().command {
        Commands::Bench {
            threads,
            dist,
            warmup,
            duration,
            report,
        } => {
            let topology: Vec<_> = (0..threads).collect();

            for addr in topology.iter() {
                run_server(
                    *addr,
                    topology.clone(),
                    dist,
                    ctx.clone(),
                    throughput.clone(),
                );
            }

            let mut total_writes_so_far = 0;

            std::thread::sleep(Duration::from_secs(warmup));

            throughput.store(0, Ordering::SeqCst);
            let start_time = Instant::now();
            let mut time_last_interval = start_time;

            loop {
                if start_time.elapsed().as_secs_f64() >= duration as f64 {
                    break;
                }

                std::thread::sleep(Duration::from_secs(1));

                if report {
                    let writes_this_interval = throughput.swap(0, Ordering::SeqCst);
                    let puts =
                        writes_this_interval as f64 / time_last_interval.elapsed().as_secs_f64();
                    time_last_interval = Instant::now();
                    println!("{puts}");

                    total_writes_so_far += writes_this_interval;
                }
            }

            total_writes_so_far += throughput.swap(0, Ordering::SeqCst);
            let puts = total_writes_so_far as f64 / start_time.elapsed().as_secs_f64();

            println!("{puts}");
        }
    }
}
