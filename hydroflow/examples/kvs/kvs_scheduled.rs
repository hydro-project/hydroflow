use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
    time::Duration,
};

use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
use hydroflow::{
    lang::{
        collections::Single,
        lattice::{ord::MaxRepr, LatticeRepr, Merge},
    },
    scheduled::{graph::Hydroflow, graph_ext::GraphExt, handoff::VecHandoff},
    tl,
};

use crate::common::{spawn, Clock, ClockedDataRepr, ClockedUpdateRepr, Message};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct ActorId(u64);

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
                let mut hf = Hydroflow::default();

                let (ticks_port, ticks_recv) = hf.make_edge::<_, VecHandoff<u64>>("ticks");
                let z_send = hf.add_channel_input("ticks handoff", ticks_port);

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

                let (writes_port, q_recv) = hf.make_edge("writes");
                hf.add_input_from_stream::<_, Option<_>, VecHandoff<_>, _>(
                    "writes",
                    writes_port,
                    receiver.map(Some),
                );

                let (reads_send, reads_recv) = hf
                    .make_edge::<_, VecHandoff<(K, futures::channel::oneshot::Sender<(Clock, V)>)>>(
                        "reads",
                    );
                let (incoming_batches_send, incoming_batches_recv) =
                    hf.make_edge::<_, VecHandoff<(usize, u64, Vec<(K, V)>)>>("received_batches");
                let (writes_send, writes_recv) =
                    hf.make_edge::<_, VecHandoff<(K, V)>>("write_requests");

                let mut current_batch: HashMap<K, <MaxRepr<V> as LatticeRepr>::Repr> =
                    HashMap::new();

                hf.add_subgraph(
                    "write_handler",
                    tl!(writes_recv, ticks_recv),
                    tl!(),
                    move |_ctx, tl!(writes_recv, ticks_recv), tl!()| {
                        for (k, v) in writes_recv.take_inner() {
                            <MaxRepr<V> as Merge<MaxRepr<V>>>::merge(
                                current_batch.entry(k).or_default(),
                                v,
                            );
                        }

                        // If we've ticked this epoch, then flush the batch.
                        if let Some(epoch) = ticks_recv.take_inner().into_iter().next() {
                            let batch: Vec<_> = current_batch.drain().collect();

                            for sender in senders.iter_mut().skip(1) {
                                sender
                                    .try_send(Message::Batch((id, epoch), batch.clone()))
                                    .unwrap();
                            }
                            senders[0]
                                .try_send(Message::Batch((id, epoch), batch))
                                .unwrap();
                        }
                    },
                );

                let mut sets = VecDeque::new();
                let mut batches = VecDeque::new();
                let mut gets = VecDeque::new();
                hf.add_subgraph(
                    "demultiplexer",
                    tl!(q_recv),
                    tl!(writes_send, incoming_batches_send, reads_send),
                    move |_ctx,
                          tl!(q_recv),
                          tl!(writes_send, incoming_batches_send, reads_send)| {
                        for v in q_recv.take_inner() {
                            match v {
                                Message::Set(k, v) => {
                                    sets.push_back((k, v));
                                }
                                Message::Batch((id, epoch), batch) => {
                                    batches.push_back((id, epoch, batch))
                                }
                                Message::Get(k, cb) => {
                                    gets.push_back((k, cb));
                                }
                            }
                        }

                        writes_send.give(std::mem::take(&mut sets));
                        incoming_batches_send.give(std::mem::take(&mut batches));
                        reads_send.give(std::mem::take(&mut gets));
                    },
                );

                let mut state = HashMap::new();
                hf.add_subgraph(
                    "read_handler",
                    tl!(reads_recv, incoming_batches_recv),
                    tl!(),
                    move |_ctx, tl!(reads_recv, incoming_batches_recv), tl!()| {
                        for (epoch, id, mut batch) in incoming_batches_recv.take_inner() {
                            for (k, v) in batch.drain(..) {
                                <ClockedDataRepr<V> as Merge<ClockedUpdateRepr<V>>>::merge(
                                    state.entry(k).or_default(),
                                    (Single((epoch, id)), v),
                                );
                            }
                        }

                        for (k, cb) in reads_recv.take_inner() {
                            if let Some(v) = state.get(&k) {
                                cb.send(v.clone()).unwrap();
                            }
                        }
                    },
                );

                hf.run_async().await.unwrap();
            })
        },
    )
}
