use std::{collections::HashMap, time::Duration};

use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    SinkExt, StreamExt,
};
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
};

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

#[derive(Clone)]
pub struct Kvs<K, V>
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
    pub fn new(workers: u64) -> Self {
        let senders = spawn_threads::<K, V>(workers);

        Kvs {
            senders,
            round_robin: 0,
        }
    }

    pub async fn set(&mut self, k: K, v: V) {
        let receiver = self.round_robin % self.senders.len();
        self.round_robin += 1;
        self.senders[receiver]
            .send(Message::Set(k, v))
            .await
            .unwrap();
    }

    pub async fn get(&mut self, k: K) -> Option<(Clock, V)> {
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
