#![feature(never_type)]

extern crate hdrhistogram;

use std::{
    str::FromStr,
    time::{Duration, Instant},
};

use clap::{ArgEnum, Parser};
use hdrhistogram::Histogram;
use hydroflow::tokio;
use rand::{prelude::Distribution, Rng};
use zipf::ZipfDistribution;

use crate::kvs::Kvs;

mod kvs;

#[derive(ArgEnum, Clone)]
enum Dist {
    Uniform(usize),
    Zipf(ZipfDistribution),
}

impl Dist {
    fn sample<R: rand::Rng>(&mut self, rng: &mut R) -> usize {
        match self {
            Self::Uniform(n) => rng.gen_range(0..*n),
            Self::Zipf(d) => d.sample(rng),
        }
    }

    fn uniform(n: usize) -> Self {
        Self::Uniform(n)
    }

    fn zipf(n: usize, theta: f64) -> Self {
        Self::Zipf(ZipfDistribution::new(n, theta).unwrap())
    }
}

impl FromStr for Dist {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut contents = s.split(',');
        match contents
            .next()
            .ok_or("type of distribution is required (uniform, zipf)")?
        {
            "uniform" => {
                let n: usize = contents
                    .next()
                    .ok_or("require a number of keys parameter")?
                    .parse()
                    .map_err(|e| format!("{}", e))?;

                Ok(Dist::uniform(n))
            }
            "zipf" => {
                let n: usize = contents
                    .next()
                    .ok_or("require a number of keys parameter")?
                    .parse()
                    .map_err(|e| format!("{}", e))?;

                let theta: f64 = contents
                    .next()
                    .ok_or("require a theta parameter")?
                    .parse()
                    .map_err(|e| format!("{}", e))?;

                Ok(Dist::zipf(n, theta))
            }
            _ => Err("invalid distribution".into()),
        }
    }
}

#[derive(Parser)]
struct Args {
    #[clap(long)]
    read_percentage: f64,
    #[clap(long)]
    num_kvs_workers: u64,
    #[clap(long)]
    num_benchmark_workers: u64,
    #[clap(long)]
    dist: Dist,
    #[clap(long)]
    run_seconds: u64,
}

fn main() {
    let args = Args::parse();

    let kvs = Kvs::new(args.num_kvs_workers);
    let run_duration = Duration::from_millis(10 * 1000);

    let read_percentage = args.read_percentage;
    let bench_workers = args.num_benchmark_workers;

    let start = Instant::now();
    let handles: Vec<_> = (0..bench_workers)
        .map(|_| {
            let mut kvs = kvs.clone();
            let mut distribution = args.dist.clone();
            std::thread::spawn(move || {
                let mut i = 0;
                let mut rng = rand::thread_rng();
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                let mut writes_hist = Histogram::<u64>::new(2).unwrap();
                let mut reads_hist = Histogram::<u64>::new(2).unwrap();
                rt.block_on(async move {
                    while start.elapsed() < run_duration {
                        let before = Instant::now();
                        if rng.gen::<f64>() < read_percentage {
                            kvs.get(format!("key{}", distribution.sample(&mut rng)))
                                .await;
                            reads_hist
                                .record(before.elapsed().as_micros().try_into().unwrap())
                                .unwrap();
                        } else {
                            kvs.set(
                                format!("key{}", distribution.sample(&mut rng)),
                                format!("bar{}", i),
                            )
                            .await;
                            writes_hist
                                .record(before.elapsed().as_micros().try_into().unwrap())
                                .unwrap();
                        }
                        i += 1;
                    }

                    (writes_hist, reads_hist)
                })
            })
        })
        .collect();

    let mut writes_hist = Histogram::<u64>::new(2).unwrap();
    let mut reads_hist = Histogram::<u64>::new(2).unwrap();
    for handle in handles {
        let (writes, reads) = handle.join().unwrap();
        writes_hist.add(writes).unwrap();
        reads_hist.add(reads).unwrap();
    }

    println!(
        "ops={}",
        writes_hist.len() + reads_hist.len(),
        // writes_hist.mean(),
        // writes_hist.value_at_quantile(0.99),
        // writes_hist.value_at_quantile(0.999),
        // reads_hist.mean(),
        // reads_hist.value_at_quantile(0.99),
        // reads_hist.value_at_quantile(0.999),
    );
}
