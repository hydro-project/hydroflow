use super::KVSRequest;
use super::KVSResponse;
use crate::util::bounded_broadcast_channel;
use futures::SinkExt;
use hydroflow::util::deserialize_from_bytes;
use hydroflow::util::serialize_to_bytes;
use hydroflow::{hydroflow_syntax, util::tcp_bytes};
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::rc::Rc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::task;
use tokio_stream::StreamExt;

use crate::MyMVReg;
use crate::MyVClock;
use crdts::ctx::AddCtx;
use crdts::CmRDT;
use crdts::CvRDT;
use hydroflow::util::deserialize_from_bytes2;
use tokio::select;

pub async fn run_server(server_addr: String, peers: Vec<String>) {
    println!("tid server: {}", palaver::thread::gettid());

    let ctx = tmq::Context::new();

    let (transducer_to_peers_tx, _) = bounded_broadcast_channel::<KVSRequest>(500000);

    let (client_to_transducer_tx, client_to_transducer_rx) =
        hydroflow::util::unbounded_channel::<(KVSRequest, Vec<u8>)>();
    let (transducer_to_client_tx, mut transducer_to_client_rx) =
        hydroflow::util::bounded_channel::<(KVSResponse, Vec<u8>)>(500000);

    // {
    //     // let pub_socket = ctx.socket(zmq::PUB).unwrap();
    //     // pub_socket.bind(&format!("tcp://{}", addr)).unwrap();

    //     // std::thread::spawn({
    //     //     let ctx = ctx.clone();
    //     //     let peers = peers.clone();

    //     //     move || {
    //     //         let sub_socket = ctx.socket(zmq::SUB).unwrap();
    //     //         sub_socket.set_subscribe(&[]).unwrap();

    //     //         for peer in peers.iter() {
    //     //             sub_socket.connect(&format!("tcp://{}", peer)).unwrap();
    //     //         }

    //     //         loop {
    //     //             let raw_msg = sub_socket.recv_msg(0).unwrap();
    //     //             let req: KVSRequest = deserialize_from_bytes2(&raw_msg);

    //     //             client_to_transducer_tx.send((req, addr)).unwrap();
    //     //         }
    //     //     }
    //     // });

    //     let ctx = tmq::Context::new();

    //     tokio::spawn({
    //         let mut router_socket = tmq::router(&ctx).bind(&format!("tcp://{}", addr)).unwrap();

    //         async move {
    //             while let Some(x) = router_socket.next().await {
    //                 let x = x.unwrap();

    //                 println!("{x:?}");
    //             }
    //         }
    //     });

    //     let mut dealer_socket = tmq::dealer(&ctx)
    //         .connect(&format!("tcp://{}", addr))
    //         .unwrap();

    //     dealer_socket.send(vec!["meme1", "meme2"]).await.unwrap();

    //     // std::thread::sleep(Duration::from_secs(1));

    //     tokio::time::sleep(Duration::from_secs(100)).await;

    //     return;
    // }

    // let client_listener = TcpListener::bind(addr).await.unwrap();

    let localset = tokio::task::LocalSet::new();

    // Handle incoming peer-to-peer communication
    // let f1 = localset.run_until(async {
    //     task::spawn_local(async move {
    //         loop {
    //             let (stream, _) = batch_listener.accept().await.unwrap();
    //             stream.set_nodelay(true).unwrap();

    //             let (_, mut inbound) = tcp_bytes(stream);

    //             task::spawn_local({
    //                 let peer_to_transducer_tx = peer_to_transducer_tx.clone();

    //                 async move {
    //                     while let Some(payload) = inbound.next().await {
    //                         let payload: KVSBatch = deserialize_from_bytes(payload.unwrap());
    //                         peer_to_transducer_tx.send(payload).unwrap();
    //                     }
    //                 }
    //             });
    //         }
    //     })
    //     .await
    //     .unwrap()
    // });

    // let clients = Rc::new(RefCell::new(HashMap::new()));

    // Handle incoming messages from clients or peers
    // let f2 = localset.run_until(async {
    //     let clients = clients.clone();

    //     task::spawn_local(async move {
    //         loop {
    //             let (stream, addr) = client_listener.accept().await.unwrap();
    //             stream.set_nodelay(true).unwrap();

    //             let (outbound, mut inbound) = tcp_bytes(stream);

    //             clients
    //                 .borrow_mut()
    //                 .insert(addr.clone(), Rc::new(RefCell::new(outbound)));

    //             task::spawn_local({
    //                 let client_to_transducer_tx = client_to_transducer_tx.clone();

    //                 async move {
    //                     while let Some(payload) = inbound.next().await {
    //                         if let Ok(payload) = payload {
    //                             let payload: KVSRequest = deserialize_from_bytes(payload);
    //                             client_to_transducer_tx.send((payload, addr)).unwrap();
    //                         }
    //                     }
    //                 }
    //             });
    //         }
    //     })
    //     .await
    //     .unwrap()
    // });

    let f2 = localset.run_until({
        let ctx = ctx.clone();
        let addr = server_addr.clone();
    
        async {
            task::spawn_local({


                async move {
                    println!("binding to: {addr:?}");
                    let mut router_socket = tmq::router(&ctx).bind(&addr).unwrap();

                    loop {
                        select! {
                            Some(x) = router_socket.next() => {
                                let x = x.unwrap();
                                assert_eq!(x.0.len(), 2);

                                let routing_id = Vec::from(&x.0[0][..]);
                                let payload: KVSRequest = deserialize_from_bytes2(&x.0[1]);
                                client_to_transducer_tx.send((payload, routing_id)).unwrap();
                            },
                            Some(x) = transducer_to_client_rx.next() => {
                                router_socket.send(vec![x.1, serialize_to_bytes(x.0).to_vec()]).await.unwrap();
                            }
                        }
                    }
                }
            })
            .await
            .unwrap()
    }});

    // Handle outgoing messages to clients
    // let f3 = localset.run_until(async {
    //     let clients = clients.clone();

    //     task::spawn_local(async move {
    //         while let Some((req, addr)) = transducer_to_client_rx.next().await {
    //             let outbound = clients.borrow().get(&addr).unwrap().clone();

    //             let _ = outbound.borrow_mut().send(serialize_to_bytes(req)).await;
    //         }
    //     })
    //     .await
    //     .unwrap()
    // });

    // Wait for other servers to set up their listening tcp sockets so the subsequent connect() calls will not fail.
    // Terrible hack, not sure of a better way to do this.
    // tokio::time::sleep(Duration::from_secs(1)).await;

    // Handle outgoing peer-to-peer communication

    let f4 = localset.run_until({
        let ctx = ctx.clone();
        let transducer_to_peers_tx = transducer_to_peers_tx.clone();
        async move {
            for peer in peers {
                let mut socket = tmq::dealer(&ctx).connect(&peer).unwrap();

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
        Duration::from_millis(1000),
    ));

    // `PollSender` adapts the send half of the bounded channel into a `Sink`.
    let transducer_to_client_tx = tokio_util::sync::PollSender::new(transducer_to_client_tx);

    #[derive(Clone, Eq, PartialEq, Debug)]
    enum ValueOrReg {
        Value(u64),
        Reg(MyMVReg),
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

        maxvclock_puts = cross_join::<'tick, 'tick>();

        put_tee = tee();

        client_input[puts]
            // -> inspect(|(key, (x, a))| if let ValueOrReg::Value(_) = x {println!("{addr}:{:5}: puts-into-crossjoin: {key:?}, {x:?}, {a:?}", context.current_tick())})
            -> put_tee;

        max_vclock = put_tee
            -> map(|(_key, x)| (0, x)) // convert group-by to fold.
            -> group_by::<'static, u64, MyVClock>(MyVClock::default, |accum: &mut MyVClock, (value_or_reg, _addr)| {
                match value_or_reg {
                    ValueOrReg::Value(_) => {
                        accum.apply(accum.inc(server_addr.clone())); // TODO: this is a hack, need to figure out what to call self.
                    },
                    ValueOrReg::Reg(reg) => {
                        accum.merge(reg.read_ctx().add_clock);
                    },
                }
            })
            // -> inspect(|x| println!("{addr}:{:5}: maxvclock-into-crossjoin: {x:?}", context.current_tick()))
            -> map(|x| x.1);

        max_vclock  -> [0]maxvclock_puts;
        put_tee     -> [1]maxvclock_puts;

        broadcast_or_store = maxvclock_puts
            // -> inspect(|x| println!("{addr}:{:5}: output of maxvclock_puts: {x:?}", context.current_tick()))
            -> demux(|(clock, (key, (value_or_reg, response_addr))): (MyVClock, (u64, (ValueOrReg, Vec<u8>))), var_args!(broadcast, store)| {
                match value_or_reg {
                    ValueOrReg::Value(value) => {
                        let mut reg = MyMVReg::default();

                        let ctx = AddCtx {
                            dot: clock.dot(server_addr.clone()),
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

        // broadcast out locally generated changes to other nodes.
        broadcast_or_store[broadcast]
            // -> next_tick() // hack: buffer operator doesn't seem to compile without this.
            -> buffer(timer)
            -> group_by::<'tick>(MyMVReg::default, |accum: &mut MyMVReg, reg: MyMVReg| {
                // If multiple writes have hit the same key then they can be merged before sending.
                accum.merge(reg);
            })
            -> map(|(key, reg)| KVSRequest::Gossip{key, reg} )
            // -> inspect(|x| println!("{addr}:{:5}: sending to peers: {x:?}", context.current_tick()))
            -> for_each(|x| { transducer_to_peers_tx.send(x).unwrap(); });

        // join for lookups
        lookup = join::<'tick, 'tick>();

        bump_merge = merge(); // group_by does not re-emit the last value every tick so need to feed in a bottom value to get it to do that.

        gets = client_input[gets] -> tee();

        gets
            -> map(|x| x)
            -> map(|(key, addr)| (key, (MyMVReg::default(), Some(addr)))) // Nasty hack to get the group_by to emit another entry at the righ
            -> bump_merge;

        put_ack_tee = tee();

        broadcast_or_store[store] -> put_ack_tee;

        transducer_to_client_tx_merge = merge();

        put_ack_tee
            -> map(|x| x)
            -> filter(|x| x.1.1.is_some())
            -> map(|(key, (_reg, addr))| (KVSResponse::PutResponse{key}, addr.unwrap()))
            -> transducer_to_client_tx_merge;

        put_ack_tee
            -> bump_merge;

        bump_merge
            -> map(|x| x)
            -> group_by::<'static, u64, MyMVReg>(MyMVReg::default, |accum: &mut MyMVReg, (reg, _addr): (MyMVReg, Option<Vec<u8>>)| {
                accum.merge(reg);
            })
            // -> inspect(|x| println!("{addr}:{:5}: stores-into-lookup: {x:?}", context.current_tick()))
            -> [0]lookup;

        // Feed gets into the join to make them do the actual matching.
        gets
            // -> inspect(|x| println!("{addr}:{:5}: gets-into-lookup: {x:?}", context.current_tick()))
            -> [1]lookup;

        // Send get results back to user
        lookup
            -> map(|x| x)
            -> map(|(key, (reg, addr))| (KVSResponse::GetResponse{key, reg}, addr))
            // -> inspect(|x| println!("{addr}:{:5}: Response to client: {x:?}", context.current_tick()))
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

    futures::join!(f2, f4, f5);
}
