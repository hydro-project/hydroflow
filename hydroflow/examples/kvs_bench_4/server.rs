use super::KVSRequest;
use super::KVSResponse;
use crate::util::bounded_broadcast_channel;
use futures::SinkExt;
use hydroflow::hydroflow_syntax;
use hydroflow::util::serialize_to_bytes;
use std::time::Duration;
use tokio::task;
use tokio_stream::StreamExt;

use crate::buffer_pool::AutoReturnBuffer;
use crate::buffer_pool::BufferPool;
use crate::MyRegType;
use crate::ValueType;
use bytes::Bytes;
use bytes::BytesMut;
use crdts::CmRDT;
use crdts::CvRDT;
use crdts::GSet;
use crdts::LWWReg;
use hydroflow::lang::lattice::crdts::EMaxReg;
use hydroflow::lang::lattice::crdts::EVClock;
use hydroflow::util::deserialize_from_bytes2;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::cell::RefCell;
use std::cell::RefMut;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use tmq::Context;
use tokio::select;

pub fn run_server(
    gossip_addr: usize,
    topology: Vec<usize>,
    dist: f64,
    ctx: Context,
    throughput: Arc<AtomicUsize>,
) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let _guard = rt.enter();

        rt.block_on(async {

            let buffer_pool = BufferPool::create_buffer_pool();

            type MyRegType2 = LWWReg<AutoReturnBuffer, u128>;

            #[derive(PartialEq, Eq, Clone, Debug)]
            pub enum KVSRequest2 {
                Put { key: u64, value: AutoReturnBuffer },
                Get { key: u64 },
                Gossip { key: u64, reg: MyRegType2 },
            }

            #[derive(PartialEq, Eq, Clone, Debug)]
            pub enum KVSResponse2 {
                PutResponse { key: u64 },
                GetResponse { key: u64, reg: MyRegType2 },
            }

            // println!("tid server: {}", palaver::thread::gettid());

            let (transducer_to_peers_tx, _) = bounded_broadcast_channel::<(u64, MyRegType2)>(500000);

            let (client_to_transducer_tx, client_to_transducer_rx) =
                hydroflow::util::unbounded_channel::<(KVSRequest2, Vec<u8>)>();
            let (transducer_to_client_tx, mut _transducer_to_client_rx) =
                hydroflow::util::bounded_channel::<(KVSResponse2, Vec<u8>)>(500000);


            // simualate some initial requests, then on each ack, send a new request.

            // {
            //     let mut rng = StdRng::from_entropy();

            //     let dist = rand_distr::Zipf::new(1_000_000, dist).unwrap();

            //     client_to_transducer_tx.send((KVSRequest::Put {
            //         key: 7,
            //         value: ValueType { data: [0; 1024] },
            //     }, Vec::new())).unwrap();

            //     client_to_transducer_tx.send((KVSRequest::Put {
            //         key: 7,
            //         value: ValueType { data: [0; 1024] },
            //     }, Vec::new())).unwrap();

            //     client_to_transducer_tx.send((KVSRequest::Get {
            //         key: 7,
            //     }, Vec::new())).unwrap();
            // }

            let localset = tokio::task::LocalSet::new();

            let f3 = localset.run_until({
                let ctx = ctx.clone();
                let buffer_pool = buffer_pool.clone();

                async {
                    task::spawn_local({

                        async move {
                            // println!("binding gossip to: {gossip_addr:?}");
                            let mut router_socket = tmq::router(&ctx).bind(&format!("inproc://S{gossip_addr}")).unwrap();

                            loop {
                                select! {
                                    Some(x) = router_socket.next() => {
                                        let x = x.unwrap();
                                        assert_eq!(x.0.len(), 2);

                                        let routing_id = Vec::from(&x.0[0][..]);
                                        let payload: KVSRequest = deserialize_from_bytes2(&x.0[1]);
                                        let payload = match payload {
                                            KVSRequest::Put { key, value } => KVSRequest2::Put {
                                                key,
                                                value: BufferPool::get_from_buffer_pool(buffer_pool.clone())
                                            },
                                            KVSRequest::Get { key } => KVSRequest2:: Get { key },
                                            KVSRequest::Gossip { key, reg } =>

                                                KVSRequest2::Gossip {
                                                    key,
                                                    reg: MyRegType2 { val: BufferPool::get_from_buffer_pool(buffer_pool.clone()), marker: reg.marker }
                                                }
                                            ,
                                        };
                                        client_to_transducer_tx.send((payload, routing_id)).unwrap();
                                    },
                                }
                            }
                        }
                    })
                    .await
                    .unwrap()
            }});

            // Handle outgoing peer-to-peer communication
            let f4 = localset.run_until({
                let ctx = ctx.clone();
                let transducer_to_peers_tx = transducer_to_peers_tx.clone();

                async move {
                    if topology.len() > 1 {
                        for addr in &topology {
                            if *addr != gossip_addr {
                                let mut socket = tmq::dealer(&ctx).connect(&format!("inproc://S{addr}")).unwrap();

                                task::spawn_local({
                                    let mut transducer_to_peers_rx = transducer_to_peers_tx.subscribe();

                                    async move {
                                        while let Ok((key, reg)) = transducer_to_peers_rx.recv().await {

                                            let req = KVSRequest::Gossip {
                                                key,
                                                reg: LWWReg {
                                                    val: ValueType{ data: reg.val[..].try_into().unwrap() },
                                                    marker: reg.marker,
                                                }
                                            };

                                            socket
                                                    .send(vec![serialize_to_bytes(req).to_vec()])
                                                    .await
                                                    .unwrap();
                                            }
                                    }
                                });
                            }
                        }
                    } else {
                        task::spawn_local({
                            let mut transducer_to_peers_rx = transducer_to_peers_tx.subscribe();

                            async move {
                                loop {
                                     transducer_to_peers_rx.recv().await.unwrap();
                                }
                            }
                        });
                    }
                }
            });

            let timer = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
                Duration::from_millis(100),
            ));

            // `PollSender` adapts the send half of the bounded channel into a `Sink`.
            let transducer_to_client_tx = tokio_util::sync::PollSender::new(transducer_to_client_tx);

            let mut rng = rand::rngs::SmallRng::from_entropy();
            let dist = rand_distr::Zipf::new(1_000_000, dist).unwrap();
            let dist_uniform = rand_distr::Uniform::new(u64::MIN, u64::MAX);
            let mut buff = BytesMut::with_capacity(1024).freeze();
            let x = Arc::new(1);

            let mut df = hydroflow_syntax! {

                hack_iter = batch_iter_fn(2000, move || {
                    // let mut buff = BytesMut::with_capacity(0);
                    // buff.resize(1024, 0);
                    // let mut r = rng.sample(dist_uniform) as u64;
                    // for i in 0..8 {
                    //     buff[i] = (r % 256) as u8;
                    //     r /= 256;
                    // }

                    let key = rng.sample(dist) as u64;

                    (KVSRequest2::Put {
                        key,
                        value: BufferPool::get_from_buffer_pool(buffer_pool.clone())
                    }, Vec::new())
                });

                my_merge = merge();

                hack_iter -> my_merge;
                source_stream(client_to_transducer_rx) -> my_merge;

                client_input = my_merge
                    -> demux(|(req, addr): (KVSRequest2, Vec<u8>), var_args!(gets, broadcast, store)| {
                        match req {
                            KVSRequest2::Put {key, value} => {
                                throughput.fetch_add(1, Ordering::SeqCst);
                                let marker = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();

                                broadcast.give((key, MyRegType2 {
                                    val: value.clone(),
                                    marker,
                                }));
                                store.give((key, MyRegType2 {
                                    val: value,
                                    marker,
                                }));
                            },
                            KVSRequest2::Gossip {key, reg} => {
                                store.give((key, reg))
                            },
                            KVSRequest2::Get {key} => gets.give((key, addr)),
                        }
                    });

                // broadcast out locally generated changes to other nodes.
                client_input[broadcast]
                    -> buffer(timer)
                    -> group_by::<'tick>(|| Option::<MyRegType2>::None, |accum: &mut Option<MyRegType2>, reg: MyRegType2| {
                        // If multiple writes have hit the same key then they can be merged before sending.
                        if accum.is_none() {
                            *accum = Some(reg);
                        } else {
                            accum.as_mut().unwrap().merge(reg);
                        }
                    })
                    -> filter_map(|(key, reg)| reg.map(|x| (key, x)))
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: sending to peers: {x:?}", context.current_tick()))
                    -> for_each(|x| { transducer_to_peers_tx.send(x).unwrap(); });

                // join for lookups
                lookup = join_hack::<'static, 'tick>();

                client_input[store]
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: stores-into-lookup: {x:?}", context.current_tick()))
                    -> [0]lookup;

                // Feed gets into the join to make them do the actual matching.
                client_input[gets]
                    -> map(|(key, addr)| {
                        let mut gset = GSet::new();
                        gset.insert(addr);
                        (key, gset)
                    })
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: gets-into-lookup: {x:?}", context.current_tick()))
                    -> [1]lookup;

                // Send get results back to user
                lookup
                    -> map(|(key, (reg, addr))| {
                        addr.read().into_iter().map(move |x| {
                            (KVSResponse2::GetResponse{key: key.clone(), reg: reg.clone()}, x.clone())
                        })
                    })
                    -> flatten()
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: Response to client: {x:?}", context.current_tick()))
                    -> dest_sink(transducer_to_client_tx);

            };

            let _serde_graph = df
                .serde_graph()
                .expect("No graph found, maybe failed to parse.");

            // println!("{}", _serde_graph.to_mermaid());

            let f5 = df.run_async();

            futures::join!(f3, f4, f5);
        });
    });
}
