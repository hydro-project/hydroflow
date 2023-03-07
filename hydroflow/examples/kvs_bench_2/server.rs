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
use crdts::CmRDT;
use crdts::CvRDT;
use crdts::GSet;
use hydroflow::lang::lattice::crdts::EAddCtx;
use hydroflow::util::deserialize_from_bytes2;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::Distribution;
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
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let _guard = rt.enter();

        rt.block_on(async {

            // println!("tid server: {}", palaver::thread::gettid());

            let (transducer_to_peers_tx, _) = bounded_broadcast_channel::<KVSRequest>(500000);

            let (client_to_transducer_tx, client_to_transducer_rx) =
                hydroflow::util::unbounded_channel::<(KVSRequest, Vec<u8>)>();
            let (transducer_to_client_tx, mut transducer_to_client_rx) =
                hydroflow::util::bounded_channel::<(KVSResponse, Vec<u8>)>(500000);


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

            {
                let mut rng = StdRng::from_entropy();

                let dist = rand_distr::Zipf::new(1_000_000, dist).unwrap();

                for _i in 0..100 {
                    client_to_transducer_tx.send((KVSRequest::Put {
                        key: dist.sample(&mut rng) as u64,
                        value: ValueType { data: [0; 1024] },
                    }, Vec::new())).unwrap();
                }
            }

            let localset = tokio::task::LocalSet::new();

            let f2 = localset.run_until({
                let client_to_transducer_tx = client_to_transducer_tx.clone();

                async {
                    task::spawn_local({

                        async move {

                            let mut rng = StdRng::from_entropy();

                            let dist = rand_distr::Zipf::new(1_000_000, dist).unwrap();

                            loop {
                                select! {
                                    Some(_) = transducer_to_client_rx.next() => {
                                        client_to_transducer_tx.send((KVSRequest::Put {
                                            key: dist.sample(&mut rng) as u64,
                                            value: ValueType { data: [0; 1024] },
                                        }, Vec::new())).unwrap();

                                        throughput.fetch_add(1, Ordering::SeqCst);
                                    }
                                }
                            }
                        }
                    })
                    .await
                    .unwrap()
            }});

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
                                        client_to_transducer_tx.send((payload, routing_id)).unwrap();
                                    },
                                }
                            }
                        }
                    })
                    .await
                    .unwrap()
            }});

            // Wait for other servers to set up their listening tcp sockets so the subsequent connect() calls will not fail.
            // Terrible hack, not sure of a better way to do this.
            // tokio::time::sleep(Duration::from_secs(1)).await;

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

            // let f4 = localset.run_until({
            //     let transducer_to_peers_tx = transducer_to_peers_tx.clone();
            //     async move {
            //         task::spawn_local(async move {
            //             // disable peers for now.
            //             for peer in peers {
            //                 let stream = TcpStream::connect(peer).await.unwrap();
            //                 stream.set_nodelay(true).unwrap();

            //                 let (mut outbound, _) = tcp_bytes(stream);
            //                 println!("connected to peer: {peer}");

            //                 task::spawn_local({
            //                     let mut transducer_to_peers_rx = transducer_to_peers_tx.subscribe();

            //                     async move {
            //                         while let Ok(batch) = transducer_to_peers_rx.recv().await {
            //                             outbound.send(serialize_to_bytes(batch)).await.unwrap();
            //                         }
            //                     }
            //                 });
            //             }
            //         })
            //         .await
            //         .unwrap()
            //     }
            // });

            let timer = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
                Duration::from_millis(100),
            ));

            // `PollSender` adapts the send half of the bounded channel into a `Sink`.
            let transducer_to_client_tx = tokio_util::sync::PollSender::new(transducer_to_client_tx);

            #[derive(Clone, Eq, PartialEq, Debug)]
            enum ValueOrReg {
                Value(ValueType),
                Reg(MyRegType),
            }

            let mut df = hydroflow_syntax! {

                client_input = source_stream(client_to_transducer_rx)
                    -> demux(|(req, addr): (KVSRequest, Vec<u8>), var_args!(puts, gets)| {
                        match req {
                            KVSRequest::Put {key, value} => puts.give((key, (ValueOrReg::Value(value), addr))),
                            KVSRequest::Get {key} => gets.give((key, addr)),
                            KVSRequest::Gossip {key, reg} => puts.give((key, (ValueOrReg::Reg(reg), addr))),
                        }
                    });

                vclock_stamper = stamp::<MyVClock>();

                put_tee = tee();

                client_input[puts]
                    -> put_tee;

                put_tee
                    -> map(|(_key, x)| (0, x)) // convert group-by to fold.
                    -> group_by::<'static, u64, MyVClock>(MyVClock::default, |accum: &mut MyVClock, (value_or_reg, _addr)| {
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
                    -> demux(|(clock, (key, (value_or_reg, response_addr))): (MyVClock, (u64, (ValueOrReg, Vec<u8>))), var_args!(broadcast, store)| {
                        match value_or_reg {
                            ValueOrReg::Value(value) => {
                                let mut reg = MyRegType::default();

                                let ctx = EAddCtx {
                                    dot: clock.dot(gossip_addr.clone()),
                                    clock: clock,
                                };

                                reg.apply(reg.write(value, ctx));

                                broadcast.give((key, reg.clone()));
                                store.give((key, (reg, Some(response_addr))));
                            },
                            ValueOrReg::Reg(reg) => {
                                store.give((key, (reg, None)));
                            },
                        }
                    });

                // broadcast_or_store = maxvclock_puts
                //     // -> inspect(|x| println!("{gossip_addr}:{:5}: output of maxvclock_puts: {x:?}", context.current_tick()))
                //     -> demux(|(clock, (key, (value_or_reg, response_addr))): (MyVClock, (u64, (ValueOrReg, Vec<u8>))), var_args!(broadcast, store)| {
                //         match value_or_reg {
                //             ValueOrReg::Value(value) => {
                //                 let mut reg = MyRegType::default();

                //                 let ctx = EAddCtx {
                //                     dot: clock.dot(gossip_addr.clone()),
                //                     clock: clock,
                //                 };

                //                 reg.apply(reg.write(value, ctx));

                //                 broadcast.give((key, reg.clone()));
                //                 store.give((key, (reg, Some(response_addr))));
                //             },
                //             ValueOrReg::Reg(reg) => {
                //                 store.give((key, (reg, None)));
                //             },
                //         }
                //     });

                // broadcast out locally generated changes to other nodes.
                broadcast_or_store[broadcast]
                    // -> next_tick() // hack: buffer operator doesn't seem to compile without this.
                    -> buffer(timer)
                    -> group_by::<'tick>(MyRegType::default, |accum: &mut MyRegType, reg: MyRegType| {
                        // If multiple writes have hit the same key then they can be merged before sending.
                        accum.merge(reg);
                    })
                    -> map(|(key, reg)| KVSRequest::Gossip{key, reg} )
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: sending to peers: {x:?}", context.current_tick()))
                    -> for_each(|x| { transducer_to_peers_tx.send(x).unwrap(); });

                // join for lookups
                lookup = join_hack::<'static, 'tick>();

                put_ack_tee = tee();

                broadcast_or_store[store] -> put_ack_tee;

                transducer_to_client_tx_merge = merge();

                put_ack_tee
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: put_ack_tee output: {x:?}", context.current_tick()))
                    -> map(|x| x)
                    -> filter(|x| x.1.1.is_some())
                    -> map(|(key, (_reg, addr))| (KVSResponse::PutResponse{key}, addr.unwrap()))
                    -> transducer_to_client_tx_merge;

                put_ack_tee
                    -> map(|(key, (reg, _addr))| (key, reg))
                    // -> group_by::<'static, u64, MyRegType>(MyRegType::default, |accum: &mut MyRegType, (reg, _addr): (MyRegType, Option<Vec<u8>>)| {
                    //     accum.merge(reg);
                    // })
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
                            (KVSResponse::GetResponse{key: key.clone(), reg: reg.clone()}, x.clone())
                        })
                    })
                    -> flatten()
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: Response to client: {x:?}", context.current_tick()))
                    -> transducer_to_client_tx_merge;

                transducer_to_client_tx_merge
                    // -> inspect(|x| println!("{addr}:{:5}: Response to client: {x:?}", context.current_tick()))
                    -> dest_sink(transducer_to_client_tx);
            };

            let _serde_graph = df
                .serde_graph()
                .expect("No graph found, maybe failed to parse.");

            // println!("{}", _serde_graph.to_mermaid());

            let f5 = df.run_async();

            futures::join!(f2, f3, f4, f5);
        });
    });
}
