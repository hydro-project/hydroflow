use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::Duration};

use futures::channel::mpsc::{channel, Receiver, Sender};
use hydroflow::{
    builder::{
        prelude::{BaseSurface, PullSurface, PushSurface},
        surface::pull_iter::IterPullSurface,
        HydroflowBuilder,
    },
    lang::{
        collections::Single,
        lattice::{map_union::MapUnionRepr, ord::MaxRepr, set_union::SetUnionRepr},
        tag,
    },
    scheduled::handoff::VecHandoff,
};

use crate::common::{Clock, ClockedDataRepr, ClockedUpdateRepr, Dist, Message, Sample};

#[derive(Clone)]
pub struct Kvs<K, V>
where
    K: Send + Clone,
    V: Send + Clone,
{
    senders: Vec<Sender<Message<K, V>>>,
    pub monitor: Rc<RefCell<Receiver<MonitorInfo>>>,
    round_robin: usize,
}

impl<K, V> Kvs<K, V>
where
    K: 'static + Clone + Eq + std::hash::Hash + Send + std::fmt::Debug + Default,
    V: 'static + Clone + Send + std::fmt::Debug + Ord + Default,
    Dist: Sample<K>,
{
    pub fn new(workers: u64, config: WorkerConfig) -> Self {
        let (senders, monitor) = spawn_threads::<K, V>(workers, config);

        Kvs {
            senders,
            monitor: Rc::new(RefCell::new(monitor)),
            round_robin: 0,
        }
    }

    #[allow(unused)]
    pub async fn set(&mut self, k: K, v: V) {
        self.send_round_robin(Message::Set(k, v));
    }

    #[allow(unused)]
    pub async fn get(&mut self, k: K) -> Option<(Clock, V)> {
        // TODO: We need to make sure we talk to one that is correct, but for
        // now since everyone owns everything just send a message to whoever.
        let (sender, receiver) = futures::channel::oneshot::channel();
        self.send_round_robin(Message::Get(k, sender));
        receiver.await.ok()
    }

    fn send_round_robin(&mut self, mut message: Message<K, V>) {
        let mut receiver_idx = self.round_robin % self.senders.len();
        self.round_robin += 1;
        while let Err(msg) = self.senders[receiver_idx].try_send(message) {
            receiver_idx = self.round_robin % self.senders.len();
            self.round_robin += 1;
            message = msg.into_inner();
        }
    }
}

// TODO(justin): this thing is hacky.
// Alternates between sending out a single unit and sending out nothing. The
// result of this is an iterator which, when put into a Hydroflow graph, will
// tick exactly once per quantum.
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

type Matrix<K, V> = Vec<(Receiver<Message<K, V>>, Vec<Sender<Message<K, V>>>)>;
type MessageSender<K, V> = Sender<Message<K, V>>;

// Constructs the NxN matrix of channels between the various participants in the
// KVS so they can all send each other messages.
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

// Every worker sends out information detailing its current status.
pub struct MonitorInfo {
    // Which worker this came from.
    pub from: usize,
    // How many messages this worker has so far processed.
    pub data: usize,
}

pub(crate) fn spawn<F, K, V>(
    n: u64,
    config: WorkerConfig,
    f: F,
) -> (Vec<Sender<Message<K, V>>>, Receiver<MonitorInfo>)
where
    F: 'static
        + Fn(
            usize,
            Receiver<Message<K, V>>,
            Vec<Sender<Message<K, V>>>,
            Sender<MonitorInfo>,
            WorkerConfig,
        )
        + Send
        + Clone,
    K: 'static + Send + Clone,
    V: 'static + Ord + Send + Clone,
{
    let core_ids = core_affinity::get_core_ids().unwrap();
    let mut core_idx = 1;

    // This channel is used to send back information about how many operations have
    // been processed.
    let (monitor_send, monitor_recv) = channel(1024);

    let (matrix, senders) = make_communication_matrix(n);
    for (i, (receiver, senders)) in matrix.into_iter().enumerate() {
        let f = f.clone();
        let id = core_ids[core_idx];
        core_idx = (core_idx + 1) % core_ids.len();

        let monitor = monitor_send.clone();
        let config = config.clone();
        std::thread::spawn(move || {
            core_affinity::set_for_current(id);
            f(i, receiver, senders, monitor, config);
        });
    }

    (senders, monitor_recv)
}

#[derive(Clone)]
pub struct WorkerConfig {
    pub epoch: Duration,
    pub batch_size: usize,
    pub dist: Dist,
}

fn spawn_threads<K, V>(
    workers: u64,
    config: WorkerConfig,
) -> (Vec<Sender<Message<K, V>>>, Receiver<MonitorInfo>)
where
    K: 'static + Clone + Eq + std::hash::Hash + Send + std::fmt::Debug + Default,
    V: 'static + Clone + Send + std::fmt::Debug + Ord + Default,
    Dist: Sample<K>,
{
    spawn(
        workers,
        config,
        move |id,
              _receiver: Receiver<Message<K, V>>,
              mut senders: Vec<Sender<Message<K, V>>>,
              mut monitor: Sender<MonitorInfo>,
              config: WorkerConfig| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut dist = config.dist.clone();
            rt.block_on(async move {
                let mut hf = HydroflowBuilder::default();

                let (epoch_flush_send, epoch_flush_recv) = hf.add_channel_input::<_, Option<u64>, VecHandoff<u64>>("ticks");

                // Construct the ticker.
                tokio::spawn(async move {
                    let mut tick = 0;
                    loop {
                        tokio::time::sleep(config.epoch).await;
                        epoch_flush_send.give(Some(tick));
                        epoch_flush_send.flush();
                        tick += 1;
                    }
                });

                let (system_send, system_recv) = hf.make_edge::<_, VecHandoff<Message<K, V>>, VecDeque<Message<K, V>>>("writes");

                let mut rng = rand::thread_rng();

                hf.add_subgraph(
                    "workload_generator",
                    IterPullSurface::new(PerQuantumPulser::new()).map_with_context(|ctx, _| {
                        // TODO(mingwei): figure out a nicer way to have a regularly scheduled operator.
                        ctx.waker().wake();
                        ctx.current_epoch()
                    }).map(move |_|{
                        (0..config.batch_size).map(|_| {
                            Message::Set(dist.sample(&mut rng), <V as Default>::default())
                        }).collect::<VecDeque<_>>()
                    })
                    .pull_to_push()
                    .push_to(system_send)
                );

                // Make the ownership table. Everyone owns everything.
                // let ownership = (0..0).map(|i| ((), Single(i)));
                // TODO(justin): this should be configurable.
                let ownership = (0..senders.len()).map(|i| ((), Single(i)));

                let (reads_send, reads_recv) =
                    hf.make_edge::<_, VecHandoff<(K, futures::channel::oneshot::Sender<(Clock, V)>)>, Option<(K, futures::channel::oneshot::Sender<(Clock, V)>)>>("reads");
                let (batches_send, batches_recv) =
                    hf.make_edge::<_, VecHandoff<(usize, u64, Vec<(K, V)>)>, Option<(usize, u64, Vec<(K, V)>)>>("received_batches");
                let (writes_send, writes_recv) = hf.make_edge::<_, VecHandoff<(K, V)>, Option<(K, V)>>("write_requests");

                let processed = hf.hydroflow.add_state(RefCell::new(0_usize));

                hf.add_subgraph(
                    "write_handler",
                    writes_recv
                        .inspect_with_context(move |ctx, msg| {
                            *ctx.state_ref(processed).borrow_mut() += msg.len();
                            // Send out monitoring data back to the coordinator thread.
                            let _ = monitor.try_send(MonitorInfo {
                                from: id,
                                data: *ctx.state_ref(processed).borrow(),
                            });
                        })
                        .flatten()
                        .map(|(k, v)| Single((k, v)))
                        .batch_with::<_, MapUnionRepr<tag::HASH_MAP, K, MaxRepr<V>>, MapUnionRepr<tag::SINGLE, K, MaxRepr<V>>, _>(epoch_flush_recv.flatten())
                        .flat_map(move |(epoch, batch)| {
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
                        // TODO(justin): this is morally a non-monotonic
                        // aggregation. We probably want a stratum boundary
                        // here.
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
                            senders[receiver].try_send(Message::Batch((id, epoch), batch)).unwrap();
                        }),
                );

                // TODO(justin): this is super ugly, we probably need a macro or something for this.
                hf.add_subgraph(
                    "demultiplexer",
                    system_recv.flatten().pull_to_push()
                    .partition(
                        |x| matches!(x, Message::Set(_, _)),
                        hf.start_tee()
                            .map(|msg| {
                                if let Message::Set(k, v) = msg {
                                    Some((k, v))
                                } else {
                                    unreachable!()
                                }
                            })
                            .push_to(writes_send),
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
                            .push_to(batches_send),
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
                    batches_recv
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
