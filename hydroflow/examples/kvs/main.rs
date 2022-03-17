#![feature(never_type)]

extern crate hdrhistogram;

use std::{
    collections::HashMap,
    str::FromStr,
    time::{Duration, Instant},
};

use clap::{ArgEnum, Parser};
use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    SinkExt,
};
use hdrhistogram::Histogram;
use hydroflow::{
    builder::{
        prelude::{BaseSurface, PullSurface, PushSurface},
        surface::pull_iter::IterPullSurface,
        HydroflowBuilder,
    },
    lang::{
        collections::Single,
        lattice::{
            dom_pair::DomPairRepr, map_union::MapUnionRepr, ord::MaxRepr, set_union::SetUnionRepr,
        },
        tag,
    },
    scheduled::handoff::VecHandoff,
    tokio,
};
use rand::{prelude::Distribution, Rng};
use tokio_stream::StreamExt;
use zipf::ZipfDistribution;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct ActorId(u64);

type Clock = HashMap<usize, u64>;

#[derive(Debug)]
enum Message<K, V>
where
    K: Send + Clone,
    V: Send + Clone,
{
    // A KV set request from a client.
    Set(K, V),
    // A KV get request from a client.
    Get(K, futures::channel::oneshot::Sender<(Clock, V)>),
    // A set of data that I am responsible for, sent to me by another worker.
    Batch((usize, u64), Vec<(K, V)>),
}

unsafe impl<K, V> Send for Message<K, V>
where
    K: Send + Clone,
    V: Send + Clone,
{
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
        "ops={}, mean (write)={}, p99 (write)={}, p99.9 (write)={}, mean (read)={}, p99 (read)={}, p99.9 (read)={}",
        writes_hist.len() + reads_hist.len(),
        writes_hist.mean(),
        writes_hist.value_at_quantile(0.99),
        writes_hist.value_at_quantile(0.999),
        reads_hist.mean(),
        reads_hist.value_at_quantile(0.99),
        reads_hist.value_at_quantile(0.999),
    );
}

#[derive(Clone)]
struct Kvs<K, V>
where
    K: Send + Clone,
    V: Send + Clone,
{
    senders: Vec<Sender<Message<K, V>>>,
    round_robin: usize,
}

impl<K, V> Kvs<K, V>
where
    K: 'static + Clone + Eq + std::hash::Hash + Send + std::fmt::Debug,
    V: 'static + Clone + Send + std::fmt::Debug + Ord + Default,
{
    fn new(workers: u64) -> Self {
        let senders = spawn_threads::<K, V>(workers);

        Kvs {
            senders,
            round_robin: 0,
        }
    }

    async fn set(&mut self, k: K, v: V) {
        let receiver = self.round_robin % self.senders.len();
        self.round_robin += 1;
        self.senders[receiver]
            .send(Message::Set(k, v))
            .await
            .unwrap();
    }

    async fn get(&mut self, k: K) -> Option<(Clock, V)> {
        // TODO: We need to make sure we talk to one that is correct, but for
        // now since everyone owns everything just send a message to whoever.
        let receiver_idx = self.round_robin % self.senders.len();
        self.round_robin += 1;
        let (sender, receiver) = futures::channel::oneshot::channel();
        self.senders[receiver_idx]
            .send(Message::Get(k, sender))
            .await
            .unwrap();

        receiver.await.ok()
    }
}

type Matrix<K, V> = Vec<(Receiver<Message<K, V>>, Vec<Sender<Message<K, V>>>)>;
type MessageSender<K, V> = Sender<Message<K, V>>;

fn make_communication_matrix<K, V>(n: u64) -> (Matrix<K, V>, Vec<MessageSender<K, V>>)
where
    K: Send + Clone,
    V: Ord + Send + Clone,
{
    let mut receivers = Vec::new();
    let mut senders: Vec<_> = (0..n).map(|_| Vec::new()).collect();
    let mut extra_senders = Vec::new();
    for _ in 0..n {
        let (sender, receiver) = channel(8192);
        receivers.push(receiver);
        for s in senders.iter_mut() {
            s.push(sender.clone())
        }
        extra_senders.push(sender);
    }

    (
        receivers.into_iter().zip(senders.into_iter()).collect(),
        extra_senders,
    )
}

fn spawn<F, K, V>(n: u64, f: F) -> Vec<Sender<Message<K, V>>>
where
    F: 'static + Fn(usize, Receiver<Message<K, V>>, Vec<Sender<Message<K, V>>>) + Send + Clone,
    K: 'static + Send + Clone,
    V: 'static + Ord + Send + Clone,
{
    let (matrix, senders) = make_communication_matrix(n);
    for (i, (receiver, senders)) in matrix.into_iter().enumerate() {
        let f = f.clone();
        std::thread::spawn(move || f(i, receiver, senders));
    }

    senders
}

// TODO(justin): this thing is hacky.
#[derive(Debug, Copy, Clone)]
struct PerQuantumPulser {
    on: bool,
}

impl PerQuantumPulser {
    fn new() -> Self {
        PerQuantumPulser { on: true }
    }
}

impl Iterator for PerQuantumPulser {
    type Item = ();
    fn next(&mut self) -> Option<()> {
        self.on = !self.on;
        if self.on {
            None
        } else {
            Some(())
        }
    }
}

type ClockRepr = MapUnionRepr<tag::HASH_MAP, usize, MaxRepr<u64>>;
type ClockUpdateRepr = MapUnionRepr<tag::SINGLE, usize, MaxRepr<u64>>;

type ClockedDataRepr<V> = DomPairRepr<ClockRepr, MaxRepr<V>>;
type ClockedUpdateRepr<V> = DomPairRepr<ClockUpdateRepr, MaxRepr<V>>;

fn spawn_threads<K, V>(workers: u64) -> Vec<Sender<Message<K, V>>>
where
    K: 'static + Clone + Eq + std::hash::Hash + Send + std::fmt::Debug,
    V: 'static + Clone + Send + std::fmt::Debug + Ord + Default,
{
    spawn(
        workers,
        move |id, receiver: Receiver<Message<K, V>>, mut senders: Vec<Sender<Message<K, V>>>| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let mut hf = HydroflowBuilder::default();

                let (z_send, z_recv) = hf.add_channel_input::<_, Option<u64>, VecHandoff<u64>>("ticks");

                // Construct the ticker.
                let epoch_duration = Duration::from_millis(100);
                tokio::spawn(async move {
                    let mut tick = 0;
                    loop {
                        tokio::time::sleep(epoch_duration).await;
                        z_send.give(Some(tick));
                        z_send.flush();
                        tick += 1;
                    }
                });

                let q_recv = hf.add_input_from_stream::<_, Option<_>, VecHandoff<_>, _>("writes", receiver.map(Some));

                // Make the ownership table. Everyone owns everything.
                let ownership = (0..senders.len()).map(|i| ((), Single(i)));

                let (reads_send, reads_recv) =
                    hf.make_edge::<_, VecHandoff<(K, futures::channel::oneshot::Sender<(Clock, V)>)>, Option<(K, futures::channel::oneshot::Sender<(Clock, V)>)>>("reads");
                let (x_send, x_recv) =
                    hf.make_edge::<_, VecHandoff<(usize, u64, Vec<(K, V)>)>, Option<(usize, u64, Vec<(K, V)>)>>("received_batches");
                let (y_send, y_recv) = hf.make_edge::<_, VecHandoff<(K, V)>, Option<(K, V)>>("write_requests");

                hf.add_subgraph(
                    "write_handler",
                    y_recv
                        .flatten()
                        .map(|(k, v)| Single((k, v)))
                        .batch_with::<_, MapUnionRepr<tag::HASH_MAP, K, MaxRepr<V>>, MapUnionRepr<tag::SINGLE, K, MaxRepr<V>>, _>(z_recv.flatten())
                        .flat_map(|(epoch, batch)| {
                            batch.into_iter().map(move |x| (epoch, x))
                        })
                        // TODO(justin): have a real hasher here, right now have
                        // exactly one bucket everyone gets written into.
                        .map(|(epoch, kv)| {
                            ((), (epoch, kv))
                        })
                        .half_hash_join::<_, _, _, SetUnionRepr<tag::VEC, usize>, SetUnionRepr<tag::SINGLE, usize>>(IterPullSurface::new(ownership))
                        .flat_map(|((), (epoch, (k, v)), receiver)| {
                            receiver.into_iter().map(move |i|
                                Single(((i, epoch), Single((k.clone(), v.clone()))))
                            )
                        })
                        .batch_with::<
                            _,
                            MapUnionRepr<tag::HASH_MAP, (usize, u64), SetUnionRepr<tag::VEC, (K, V)>>,
                            MapUnionRepr<tag::SINGLE, (usize, u64), SetUnionRepr<tag::SINGLE, (K, V)>>,
                            _,
                        >(IterPullSurface::new(PerQuantumPulser::new()))
                        .map(|((), batch)| batch)
                        .filter(|batch| !batch.is_empty())
                        .flatten()
                        .pull_to_push()
                        .for_each(move |((receiver, epoch), batch)| {
                            // TODO(justin): do we need to tag this with our current vector clock as well?
                            senders[receiver].try_send(Message::Batch((id, epoch), batch)).unwrap();
                        }),
                );

                // TODO(justin): this is super ugly, we probably need a macro or something for this.
                hf.add_subgraph(
                    "demultiplexer",
                    q_recv.flatten().pull_to_push().partition(
                        |x| matches!(x, Message::Set(_, _)),
                        hf.start_tee()
                            .map(|msg| {
                                if let Message::Set(k, v) = msg {
                                    Some((k, v))
                                } else {
                                    unreachable!()
                                }
                            })
                            .push_to(y_send),
                        hf.start_tee()
                        .partition(
                        |x| matches!(x, Message::Batch(_, _)),
                            hf.start_tee().map(move |msg| {
                                if let Message::Batch((id, epoch), batch) = msg {
                                    Some((id, epoch, batch))
                                } else {
                                    unreachable!()
                                }
                            })
                            .push_to(x_send),
                            hf.start_tee().map(|msg| {
                                if let Message::Get(k, sender) = msg {
                                    Some((k, sender))
                                } else {
                                    unreachable!()
                                }
                            })
                            .push_to(reads_send),
                        )
                    ),
                );

                hf.add_subgraph(
                    "read_handler",
                    reads_recv.flatten().map(
                        |(k, ch)| (k, ch)
                        ).half_hash_join::<_, _, _, ClockedDataRepr<V>, ClockedUpdateRepr<V>>(
                    x_recv
                        .flatten()
                        .flat_map(move |(id, epoch, batch)| {
                            batch.into_iter().map(
                                move |(k, v)| (k, (Single((id, epoch)), v))
                            )
                        }))
                        .pull_to_push()
                        .for_each(|(_k, sender, (clock, v))| {
                            sender.send((clock, v)).unwrap()
                        })
                );

                hf.build().run_async().await.unwrap();
            })
        },
    )
}
