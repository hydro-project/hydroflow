use super::KVSRequest;
use super::KVSResponse;
use crate::util::bounded_broadcast_channel;
use futures::SinkExt;
use hydroflow::hydroflow_syntax;
use hydroflow::util::serialize_to_bytes;
use std::time::Duration;
use tokio::task;
use tokio_stream::StreamExt;

use crate::MyRegType;
use crate::MyVClock;
use crate::ValueType;
use bytes::Bytes;
use bytes::BytesMut;
use crdts::CmRDT;
use crdts::CvRDT;
use crdts::GSet;
use hydroflow::lang::lattice::crdts::EMaxReg;
use hydroflow::lang::lattice::crdts::EVClock;
use hydroflow::util::deserialize_from_bytes2;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tmq::Context;
use tokio::select;

pub fn run_server(
    gossip_addr: usize,
    topology: Vec<usize>,
    dist: f64,
    ctx: Context,
    throughput: Arc<AtomicUsize>,
) {
    type MyRegType2 = EMaxReg<Bytes>;
    type MyVClock2 = EVClock;

    #[derive(PartialEq, Eq, Clone, Debug)]
    pub enum KVSRequest2 {
        Put { key: u64, value: Bytes },
        Get { key: u64 },
        Gossip { key: u64, reg: MyRegType2 },
    }

    #[derive(PartialEq, Eq, Clone, Debug)]
    pub enum KVSResponse2 {
        PutResponse { key: u64 },
        GetResponse { key: u64, reg: MyRegType2 },
    }

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let _guard = rt.enter();

        rt.block_on(async {

            // println!("tid server: {}", palaver::thread::gettid());

            let (transducer_to_peers_tx, _) = bounded_broadcast_channel::<KVSRequest2>(500000);

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
                                            KVSRequest::Put { key, value } => KVSRequest2::Put { key, value: BytesMut::from(&value.data[..]).freeze() },
                                            KVSRequest::Get { key } => KVSRequest2::Get { key },
                                            KVSRequest::Gossip { key, reg } => KVSRequest2::Gossip { key, reg: EMaxReg { clock: reg.read_ctx().clock, val: BytesMut::from(&reg.read().val.data[..]).freeze() } },
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
                                        while let Ok(batch) = transducer_to_peers_rx.recv().await {

                                            let batch = match batch {
                                                KVSRequest2::Put { key, value } => KVSRequest::Put { key, value: ValueType { data: value[..].try_into().unwrap() } },
                                                KVSRequest2::Get { key } => KVSRequest::Get { key },
                                                KVSRequest2::Gossip { key, reg } => KVSRequest::Gossip { key, reg: EMaxReg { clock: reg.read_ctx().clock, val: ValueType { data: reg.read().val[..].try_into().unwrap() } } },
                                            };

                                            socket
                                                    .send(vec![serialize_to_bytes(batch).to_vec()])
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

            #[derive(Clone, Eq, PartialEq, Debug)]
            enum ValueOrReg {
                Value(Bytes),
                Reg(MyRegType2),
            }

            let mut rng = rand::rngs::SmallRng::from_entropy();
            let dist = rand_distr::Zipf::new(1_000_000, dist).unwrap();

            let buff = Bytes::from_static(&[0; 1024]);

            let mut df = hydroflow_syntax! {

                hack_iter = batch_iter_fn(20, move || {
                    (KVSRequest2::Put {
                        key: rng.sample(dist) as u64,
                        value: buff.clone()
                    }, Vec::new())
                });

                // hack_iter = repeat_iter([
                //     (KVSRequest::Put {
                //         key: 0,
                //         value: ValueType {
                //             data: [0; 1024],
                //         }
                //     }, Vec::new()),
                //     (KVSRequest::Put {
                //         key: 0,
                //         value: ValueType {
                //             data: [0; 1024],
                //         }
                //     }, Vec::new()),
                //     (KVSRequest::Put {
                //         key: 0,
                //         value: ValueType {
                //             data: [0; 1024],
                //         }
                //     }, Vec::new()),
                //     (KVSRequest::Put {
                //         key: 0,
                //         value: ValueType {
                //             data: [0; 1024],
                //         }
                //     }, Vec::new())
                // ]);

                my_merge = merge();

                hack_iter -> my_merge;
                source_stream(client_to_transducer_rx) -> my_merge;

                // real_iter = source_stream(client_to_transducer_rx);

                client_input = my_merge
                    -> demux(|(req, addr): (KVSRequest2, Vec<u8>), var_args!(puts, gets)| {
                        match req {
                            KVSRequest2::Put {key, value} => {
                                throughput.fetch_add(1, Ordering::SeqCst);
                                puts.give((key, (ValueOrReg::Value(value), addr)))
                            },
                            KVSRequest2::Get {key} => gets.give((key, addr)),
                            KVSRequest2::Gossip {key, reg} => puts.give((key, (ValueOrReg::Reg(reg), addr))),
                        }
                    });

                vclock_stamper = stamp::<MyVClock2>();

                put_tee = tee();

                client_input[puts]
                    -> put_tee;

                put_tee
                    -> map(|(_key, x)| (0, x)) // convert group-by to fold.
                    -> group_by::<'static, u64, MyVClock2>(MyVClock2::default, |accum: &mut MyVClock2, (value_or_reg, _addr)| {
                        match value_or_reg {
                            ValueOrReg::Value(_) => {
                                accum.apply(accum.inc(gossip_addr.clone())); // TODO: this is a hack, need to figure out what to call self.
                            },
                            ValueOrReg::Reg(reg) => {
                                accum.merge(reg.read_ctx().clock);
                            },
                        }
                    })
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: maxvclock-into-stamper: {x:?}", context.current_tick()))
                    -> map(|x| x.1)
                    -> [0]vclock_stamper;

                put_tee
                    // -> inspect(|(key, (x, a))| if let ValueOrReg::Value(_) = x {println!("{gossip_addr}:{:5}: puts-into-stamper: {key:?}, {x:?}, {a:?}", context.current_tick())})
                    -> [1]vclock_stamper;

                broadcast_or_store = vclock_stamper
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: output of maxvclock_puts: {x:?}", context.current_tick()))
                    -> demux(|(clock, (key, (value_or_reg, response_addr))): (MyVClock2, (u64, (ValueOrReg, Vec<u8>))), var_args!(broadcast, store)| {
                        match value_or_reg {
                            ValueOrReg::Value(value) => {
                                let reg = MyRegType2::from_raw(clock, value);

                                broadcast.give((key, reg.clone()));
                                store.give((key, (reg, Some(response_addr))));
                            },
                            ValueOrReg::Reg(reg) => {
                                store.give((key, (reg, None)));
                            },
                        }
                    });

                // broadcast out locally generated changes to other nodes.
                broadcast_or_store[broadcast]
                    // -> next_tick() // hack: buffer operator doesn't seem to compile without this.
                    -> buffer(timer)
                    -> group_by::<'tick>(MyRegType2::default, |accum: &mut MyRegType2, reg: MyRegType2| {
                        // If multiple writes have hit the same key then they can be merged before sending.
                        accum.merge(reg);
                    })
                    -> map(|(key, reg)| KVSRequest2::Gossip{key, reg} )
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: sending to peers: {x:?}", context.current_tick()))
                    -> for_each(|x| { transducer_to_peers_tx.send(x).unwrap(); });

                // join for lookups
                lookup = join_hack::<'static, 'tick>();

                broadcast_or_store[store]
                    -> map(|(key, (reg, _addr))| (key, reg))
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
                    -> map(|x| x)
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
