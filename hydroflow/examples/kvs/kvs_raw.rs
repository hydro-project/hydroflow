use std::{sync::Arc, time::Duration};

use hydroflow::{
    lang::{
        collections::Single,
        lattice::{
            dom_pair::DomPairRepr, map_union::MapUnionRepr, ord::MaxRepr, LatticeRepr, Merge,
        },
        tag,
    },
    tokio::{
        self,
        sync::{
            mpsc::{channel, Receiver, Sender},
            Mutex,
        },
    },
};

use crate::common::{Clock, Message};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct ActorId(u64);

type ClockRepr = MapUnionRepr<tag::HASH_MAP, usize, MaxRepr<u64>>;
type ClockUpdateRepr = MapUnionRepr<tag::SINGLE, usize, MaxRepr<u64>>;

type DataRepr<K, V> = MapUnionRepr<tag::HASH_MAP, K, DomPairRepr<ClockRepr, MaxRepr<V>>>;
type BatchRepr<K, V> = MapUnionRepr<tag::VEC, K, MaxRepr<V>>;
type BatchUpdateRepr<K, V> = MapUnionRepr<tag::SINGLE, K, MaxRepr<V>>;
type UpdateRepr<K, V> = MapUnionRepr<tag::SINGLE, K, DomPairRepr<ClockRepr, MaxRepr<V>>>;

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
    V: Send + Clone,
{
    let mut receivers = Vec::new();
    let mut senders: Vec<_> = (0..n).map(|_| Vec::new()).collect();
    let mut extra_senders = Vec::new();
    for _ in 0..n {
        let (sender, receiver) = channel(1024);
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
    V: 'static + Send + Clone,
{
    let (matrix, senders) = make_communication_matrix(n);
    for (i, (receiver, senders)) in matrix.into_iter().enumerate() {
        let f = f.clone();
        std::thread::spawn(move || f(i, receiver, senders));
    }

    senders
}

// TODO(justin): add a stack-allocated implementation of this.
fn owners<K: std::hash::Hash>(n: u64, _v: &K) -> Vec<usize> {
    (0..n as usize).collect()
}

fn spawn_threads<K, V>(workers: u64) -> Vec<Sender<Message<K, V>>>
where
    K: 'static + Clone + Eq + std::hash::Hash + Send + std::fmt::Debug,
    V: 'static + Clone + Send + std::fmt::Debug + Ord + Default,
{
    spawn(
        workers,
        move |id, mut receiver: Receiver<Message<K, V>>, senders| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let clock: <ClockRepr as LatticeRepr>::Repr = Default::default();
                let clock = Arc::new(Mutex::new(clock));
                let mut epoch = 0_u64;

                let current_updates: <DataRepr<K, V> as LatticeRepr>::Repr = Default::default();
                let current_updates = Arc::new(Mutex::new(current_updates));

                let data: <DataRepr<K, V> as LatticeRepr>::Repr = Default::default();
                let data = Arc::new(Mutex::new(data));

                let event_updates = current_updates.clone();

                let event_loop = tokio::spawn(async move {
                    while let Some(msg) = receiver.recv().await {
                        match msg {
                            Message::Set(k, v) => {
                                let mut updates = event_updates.lock().await;
                                <DataRepr<K, V> as Merge<UpdateRepr<K, V>>>::merge(
                                    &mut updates,
                                    Single((k, (clock.lock().await.clone(), v))),
                                );
                            }
                            Message::Batch((from, epoch), mut batch) => {
                                let mut clock = clock.lock().await;
                                <ClockRepr as Merge<ClockUpdateRepr>>::merge(
                                    &mut clock,
                                    Single((from, epoch)),
                                );
                                for (k, v) in batch.drain(..) {
                                    let mut data = data.lock().await;
                                    <DataRepr<K, V> as Merge<UpdateRepr<K, V>>>::merge(
                                        &mut data,
                                        Single((k, (clock.clone(), v))),
                                    );
                                }
                            }
                            Message::Get(k, cb) => {
                                let data = data.lock().await;
                                if let Some(v) = data.get(&k) {
                                    cb.send(v.clone()).unwrap()
                                }
                            }
                        }
                    }
                });

                let epoch_duration = Duration::from_millis(100);
                tokio::spawn(async move {
                    loop {
                        tokio::time::sleep(epoch_duration).await;
                        epoch += 1;
                        let my_clock = Single((id, epoch));
                        // let mut clock = clock.lock().await;
                        // <ClockRepr as Merge<ClockUpdateRepr>>::merge(&mut clock, Single((id, epoch)));
                        let mut batches: Vec<<BatchRepr<K, V> as LatticeRepr>::Repr> =
                            (0..workers).map(|_| Default::default()).collect();

                        for (k, (mut ts, v)) in current_updates.lock().await.drain() {
                            <ClockRepr as Merge<ClockUpdateRepr>>::merge(&mut ts, my_clock);
                            // TODO(justin): save a clone here.
                            for owner in owners(workers, &k) {
                                <BatchRepr<K, V> as Merge<BatchUpdateRepr<K, V>>>::merge(
                                    &mut batches[owner],
                                    Single((k.clone(), v.clone())),
                                );
                            }
                        }

                        // TODO(justin): do this in parallel?
                        // TODO(justin): reuse the memory by keeping the vecs around?
                        for (s, batch) in senders.iter().zip(batches.into_iter()) {
                            s.send(Message::Batch((id, epoch), batch)).await.unwrap();
                        }
                    }
                });

                event_loop.await.unwrap();
            })
        },
    )
}
