use super::KVSBatch;
use super::KVSRequest;
use super::KVSResponse;
use crate::util::bounded_broadcast_channel;
use crdts::CvRDT;
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
use crdts::CmRDT;

pub async fn run_server(batch_addr: SocketAddr, client_addr: SocketAddr, peers: Vec<SocketAddr>) {
    println!("tid server: {}", palaver::thread::gettid());

    let batch_listener = TcpListener::bind(batch_addr).await.unwrap();
    let client_listener = TcpListener::bind(client_addr).await.unwrap();

    let (transducer_to_peers_tx, _) = bounded_broadcast_channel::<KVSBatch>(100);
    let (peer_to_transducer_tx, peer_to_transducer_rx) =
        hydroflow::util::unbounded_channel::<KVSBatch>();

    let (client_to_transducer_tx, client_to_transducer_rx) =
        hydroflow::util::unbounded_channel::<(KVSRequest, SocketAddr)>();
    let (transducer_to_client_tx, mut transducer_to_client_rx) =
        hydroflow::util::bounded_channel::<(KVSResponse, SocketAddr)>(50000);

    let localset = tokio::task::LocalSet::new();

    // Handle incoming peer-to-peer communication
    let f1 = localset.run_until(async {
        task::spawn_local(async move {
            loop {
                let (stream, _) = batch_listener.accept().await.unwrap();
                stream.set_nodelay(true).unwrap();

                let (_, mut inbound) = tcp_bytes(stream);

                task::spawn_local({
                    let peer_to_transducer_tx = peer_to_transducer_tx.clone();

                    async move {
                        while let Some(payload) = inbound.next().await {
                            let payload: KVSBatch = deserialize_from_bytes(payload.unwrap());
                            peer_to_transducer_tx.send(payload).unwrap();
                        }
                    }
                });
            }
        })
        .await
        .unwrap()
    });

    let clients = Rc::new(RefCell::new(HashMap::new()));

    // Handle incoming messages from clients
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

    let mut df = hydroflow_syntax! {

        my_demux = source_stream(client_to_transducer_rx)
            -> demux(|(req, addr), var_args!(puts, gets)| match req {
                KVSRequest::Put {key, value} => puts.give((key, value)),
                KVSRequest::Get {key} => gets.give((key, addr)),
            });

        puts = my_demux[puts]
            -> group_by::<u64, MyMVReg>(MyMVReg::default, |old: &mut MyMVReg, val: u64| {
                let op = old.write(val, old.read_ctx().derive_add_ctx(batch_addr));
                old.apply(op);
            })
            -> inspect(|x| println!("{batch_addr}: puts: {x:?}"))
            -> tee();

        gets = my_demux[gets]
            -> inspect(|x| println!("{batch_addr}: gets: {x:?}"));

        // Broadcast merged puts out to peers
        puts
            -> map(|(key, reg)| KVSBatch::Batch {key, reg})
            -> inspect(|x| println!("{batch_addr}: broadcasting put: {x:?}"))
            -> for_each(|x| { transducer_to_peers_tx.send(x).unwrap(); });

        // merge in batches received from other peers
        my_merge = merge();
        source_stream(peer_to_transducer_rx)
            -> map(|batch| match batch {KVSBatch::Batch {key, reg} => (key, reg)})
            -> inspect(|x| println!("{batch_addr}: incoming merge: {x:?}"))
            -> my_merge;
        puts[0] -> my_merge;

        // join PUTs and GETs by key
        lookup = join::<'static, 'tick>() -> inspect(|x| println!("{batch_addr}: join: {x:?}"));
        my_merge -> [0]lookup;
        gets -> [1]lookup;

        // every time epoch elapses, broadcast updates to anna peers
        // source_stream(timer) -> map(|x| KVSBatch::Batch {actor: batch_addr, batches: vec![(0, 0)]}) -> for_each(|x| { transducer_to_peers_tx.send(x).unwrap(); });

        lookup
            -> group_by::<'tick>(|| (MyMVReg::default(), None), |old: &mut (MyMVReg, Option<SocketAddr>), val: (MyMVReg, SocketAddr)| {
                old.0.merge(val.0);
                old.1 = Some(val.1);
            })
            -> map(|(key, (reg, addr))| (KVSResponse::Response{key, reg}, addr.unwrap()))
            -> inspect(|x| println!("{batch_addr}: Sending: {x:?}"))
            -> dest_sink(transducer_to_client_tx);
    };

    let f5 = df.run_async();

    futures::join!(f1, f2, f3, f4, f5);
}

// group_by::<u64, MyMVReg>(MyMVReg::default, |old: &mut MyMVReg, val: u64| {
//     let op = old.write(val, old.read_ctx().derive_add_ctx(batch_addr));
//     old.apply(op);
// }) ->
