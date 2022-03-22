#![feature(never_type)]

extern crate hdrhistogram;

use std::{
    str::FromStr,
    time::{Duration, Instant},
};

use clap::{ArgEnum, Parser};
use common::Clock;
use hdrhistogram::Histogram;
use hydroflow::tokio;
use rand::{prelude::Distribution, Rng};
use zipf::ZipfDistribution;

mod common;
mod kvs_compiled;
mod kvs_scheduled;

// This would ideally be a trait, but trait methods can't be async.
#[derive(Clone)]
enum KvsImplementation<K, V>
where
    K: 'static + Clone + Eq + std::hash::Hash + Send + std::fmt::Debug,
    V: 'static + Clone + Send + std::fmt::Debug + Ord + Default,
{
    Scheduled(kvs_scheduled::Kvs<K, V>),
    Compiled(kvs_compiled::Kvs<K, V>),
}

impl<K, V> KvsImplementation<K, V>
where
    K: 'static + Clone + Eq + std::hash::Hash + Send + std::fmt::Debug,
    V: 'static + Clone + Send + std::fmt::Debug + Ord + Default,
{
    async fn set(&mut self, k: K, v: V) {
        match self {
            Self::Scheduled(kvs) => kvs.set(k, v).await,
            Self::Compiled(kvs) => kvs.set(k, v).await,
        }
    }

    async fn get(&mut self, k: K) -> Option<(Clock, V)> {
        match self {
            Self::Scheduled(kvs) => kvs.get(k).await,
            Self::Compiled(kvs) => kvs.get(k).await,
        }
    }
}

impl<K, V> FromStr for KvsImplementation<K, V>
where
    K: 'static + Clone + Eq + std::hash::Hash + Send + std::fmt::Debug,
    V: 'static + Clone + Send + std::fmt::Debug + Ord + Default,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (kind, workers) = s
            .split_once(',')
            .ok_or("need to specify number of workers")?;
        let workers = workers.parse().map_err(|e| format!("{}", e))?;
        match kind {
            "scheduled" => Ok(KvsImplementation::Scheduled(kvs_scheduled::Kvs::new(
                workers,
            ))),
            "compiled" => Ok(KvsImplementation::Compiled(kvs_compiled::Kvs::new(workers))),
            _ => Err("not a valid implementation".into()),
        }
    }
}

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
    #[clap(long)]
    implementation: KvsImplementation<String, String>,
}

fn main() {
    let args = Args::parse();

    let kvs = args.implementation;
    let run_duration = Duration::from_millis(args.run_seconds * 1000);

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
