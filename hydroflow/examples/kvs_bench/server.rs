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
use crdts::ctx::AddCtx;
use crdts::CmRDT;
use crdts::CvRDT;
use crdts::VClock;

pub async fn run_server(addr: SocketAddr, peers: Vec<SocketAddr>) {
    println!("tid server: {}", palaver::thread::gettid());

    let client_listener = TcpListener::bind(addr).await.unwrap();

    let (transducer_to_peers_tx, _) = bounded_broadcast_channel::<KVSRequest>(500000);

    let (client_to_transducer_tx, client_to_transducer_rx) =
        hydroflow::util::unbounded_channel::<(KVSRequest, SocketAddr)>();
    let (transducer_to_client_tx, mut transducer_to_client_rx) =
        hydroflow::util::bounded_channel::<(KVSResponse, SocketAddr)>(500000);

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

    let clients = Rc::new(RefCell::new(HashMap::new()));

    // Handle incoming messages from clients or peers
    let f2 = localset.run_until(async {
        let clients = clients.clone();

        task::spawn_local(async move {
            loop {
                let (stream, addr) = client_listener.accept().await.unwrap();
                stream.set_nodelay(true).unwrap();

                let (outbound, mut inbound) = tcp_bytes(stream);

                clients
                    .borrow_mut()
                    .insert(addr.clone(), Rc::new(RefCell::new(outbound)));

                task::spawn_local({
                    let client_to_transducer_tx = client_to_transducer_tx.clone();

                    async move {
                        while let Some(payload) = inbound.next().await {
                            let payload: KVSRequest = deserialize_from_bytes(payload.unwrap());
                            client_to_transducer_tx.send((payload, addr)).unwrap();
                        }
                    }
                });
            }
        })
        .await
        .unwrap()
    });

    // Handle outgoing messages to clients
    let f3 = localset.run_until(async {
        let clients = clients.clone();

        task::spawn_local(async move {
            while let Some((req, addr)) = transducer_to_client_rx.next().await {
                let outbound = clients.borrow().get(&addr).unwrap().clone();

                outbound
                    .borrow_mut()
                    .send(serialize_to_bytes(req))
                    .await
                    .unwrap();
            }
        })
        .await
        .unwrap()
    });

    // Wait for other servers to set up their listening tcp sockets so the subsequent connect() calls will not fail.
    // Terrible hack, not sure of a better way to do this.
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Handle outgoing peer-to-peer communication

    let f4 = localset.run_until({
        let transducer_to_peers_tx = transducer_to_peers_tx.clone();
        async move {
            task::spawn_local(async move {
                // disable peers for now.
                for peer in peers {
                    let stream = TcpStream::connect(peer).await.unwrap();
                    stream.set_nodelay(true).unwrap();

                    let (mut outbound, _) = tcp_bytes(stream);
                    println!("connected to peer: {peer}");

                    task::spawn_local({
                        let mut transducer_to_peers_rx = transducer_to_peers_tx.subscribe();

                        async move {
                            while let Ok(batch) = transducer_to_peers_rx.recv().await {
                                outbound.send(serialize_to_bytes(batch)).await.unwrap();
                            }
                        }
                    });
                }
            })
            .await
            .unwrap()
        }
    });

    // let timer = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
    //     Duration::from_millis(1000),
    // ));

    // `PollSender` adapts the send half of the bounded channel into a `Sink`.
    let transducer_to_client_tx = tokio_util::sync::PollSender::new(transducer_to_client_tx);

    #[derive(Clone, Eq, PartialEq, Debug)]
    enum ValueOrReg {
        Value(u64),
        Reg(MyMVReg),
    }

    let mut df = hydroflow_syntax! {

        my_demux = source_stream(client_to_transducer_rx)
            -> demux(|(req, addr), var_args!(puts, gets)| {
                match req {
                    KVSRequest::Put {key, value} => puts.give((key, ValueOrReg::Value(value))),
                    KVSRequest::Get {key} => gets.give((key, addr)),
                    KVSRequest::Gossip {key, reg} => puts.give((key, ValueOrReg::Reg(reg))),
                }
            });

        my_crossjoin = cross_join::<'tick, 'tick>();

        put_tee = tee();

        my_demux[puts]
            // -> inspect(|x| println!("{addr}:{:5}: puts-in: {x:?}", context.current_tick()))
            -> put_tee;

        put_tee
            -> map(|x| x)
            -> group_by::<'static, u64, VClock<SocketAddr>>(VClock::default, |accum: &mut VClock<SocketAddr>, value_or_reg| {
                match value_or_reg {
                    ValueOrReg::Value(_) => {
                        accum.apply(accum.inc(addr));
                    },
                    ValueOrReg::Reg(reg) => {
                        accum.merge(reg.read_ctx().add_clock);
                    },
                }
            })
            -> map(|x| x.1)
            // -> inspect(|x| println!("{addr}:{:5}: puts-into-crossjoin: {x:?}", context.current_tick()))
            -> [0]my_crossjoin;

        put_tee -> [1]my_crossjoin;

        lookup = join::<'tick, 'tick>();

        my_demux2 = my_crossjoin
            -> demux(|(clock, (key, value_or_reg)): (VClock<SocketAddr>, (u64, ValueOrReg)), var_args!(broadcast, store)| {
                match value_or_reg {
                    ValueOrReg::Value(value) => {
                        let mut reg = MyMVReg::default();

                        let ctx = AddCtx {
                            dot: clock.dot(addr),
                            clock: clock,
                        };

                        reg.apply(reg.write(value, ctx));

                        broadcast.give((key, reg.clone()));
                        store.give((key, reg));
                    },
                    ValueOrReg::Reg(reg) => {
                        store.give((key, reg));
                    },
                }
            });

        // Merge all puts together
        my_merge5 = merge();
        my_tee5 = tee();

        my_demux[gets] -> my_tee5;

        my_demux2[store] -> my_merge5;
        my_tee5
            -> map(|(key, _addr)| (key, MyMVReg::default())) // Nasty hack to get the group_by to emit another entry at the right time...
            -> my_merge5;

        my_merge5
            -> group_by::<'static>(MyMVReg::default, |accum: &mut MyMVReg, reg: MyMVReg| {
                accum.merge(reg);
            })
            -> [0]lookup;

        // Broadcast ops to other peers
        my_demux2[broadcast]
            -> map(|(key, reg)| KVSRequest::Gossip {key, reg})
            // -> inspect(|x| println!("{addr}:{:5}: broadcasting put: {x:?}", context.current_tick()))
            -> for_each(|x| { transducer_to_peers_tx.send(x).unwrap(); });

        // Feed gets into the join to make them do the actual matching.
        my_tee5
            -> map(|x| x)
            // -> inspect(|x| println!("{addr}:{:5}: gets-into-lookup: {x:?}", context.current_tick()))
            -> [1]lookup;

        // Send get results back to user
        lookup
            -> map(|(key, (reg, addr))| (KVSResponse::Response{key, reg}, addr))
            // -> inspect(|x| println!("{addr}:{:5}: Response to client: {x:?}", context.current_tick()))
            -> dest_sink(transducer_to_client_tx);
    };

    let serde_graph = df
        .serde_graph()
        .expect("No graph found, maybe failed to parse.");

    println!("{}", serde_graph.to_mermaid());

    let f5 = df.run_async();

    futures::join!(f2, f3, f4, f5);
}

// group_by::<u64, MyMVReg>(MyMVReg::default, |old: &mut MyMVReg, val: u64| {
//     let op = old.write(val, old.read_ctx().derive_add_ctx(batch_addr));
//     old.apply(op);
// }) ->
