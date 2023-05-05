use crate::util::bounded_broadcast_channel;
use futures::SinkExt;
use hydroflow::hydroflow_syntax;
use std::time::Duration;
use tokio::task;
use tokio_stream::StreamExt;

use crate::buffer_pool::BufferPool;
use crate::protocol::KvsRequest;
use crate::protocol::KvsRequestDeserializer;
use crate::protocol::KvsResponse;
use crate::protocol::MyLastWriteWins;
use crate::protocol::MySetUnion;
use bincode::options;
use rand::Rng;
use rand::SeedableRng;
use serde::de::DeserializeSeed;
use serde::Serialize;
use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tmq::Context;

use lattices::bottom::Bottom;
use lattices::fake::Fake;
use lattices::ord::Max;
use lattices::set_union::SetUnionSingletonSet;
use lattices::Merge;

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

        rt.block_on(async {

            let buffer_pool = BufferPool::create_buffer_pool();

            let (transducer_to_peers_tx, _) =
                bounded_broadcast_channel::<(u64, MyLastWriteWins)>(500000);

            let (client_to_transducer_tx, client_to_transducer_rx) =
                hydroflow::util::unbounded_channel::<(KvsRequest, Vec<u8>)>();
            let (transducer_to_client_tx, mut _transducer_to_client_rx) =
                hydroflow::util::unbounded_channel::<(KvsResponse, Vec<u8>)>();

            let localset = tokio::task::LocalSet::new();

            let inbound_networking_task = localset.run_until({
                let ctx = ctx.clone();
                let buffer_pool = buffer_pool.clone();

                async {
                    task::spawn_local({

                        async move {
                            let mut router_socket = tmq::router(&ctx).bind(&format!("inproc://S{gossip_addr}")).unwrap();

                            loop {
                                if let Some(x) = router_socket.next().await {
                                    let multipart_buffer = x.unwrap();
                                    assert_eq!(multipart_buffer.0.len(), 2);

                                    let routing_id = Vec::from(&multipart_buffer.0[0][..]);

                                    let reader = std::io::Cursor::new(&multipart_buffer.0[1][..]);
                                    let mut deserializer = bincode::Deserializer::with_reader(reader, options());
                                    let req = KvsRequestDeserializer {
                                        collector: Rc::clone(&buffer_pool),
                                    }.deserialize(&mut deserializer).unwrap();

                                    client_to_transducer_tx.send((req, routing_id)).unwrap();
                                }
                            }
                        }
                    })
                    .await
                    .unwrap()
            }});

            // Handle outgoing peer-to-peer communication
            let outbound_networking_task = localset.run_until({
                let ctx = ctx.clone();
                let transducer_to_peers_tx = transducer_to_peers_tx.clone();

                // TODO: Eventually this would get moved into a hydroflow operator that would return a Bytes struct and be efficient and zero copy and etc.
                async move {
                    if topology.len() > 1 {
                        for addr in &topology {
                            if *addr != gossip_addr {
                                let mut socket = tmq::dealer(&ctx).connect(&format!("inproc://S{addr}")).unwrap();

                                task::spawn_local({
                                    let mut transducer_to_peers_rx = transducer_to_peers_tx.subscribe();

                                    async move {
                                        while let Ok((key, reg)) = transducer_to_peers_rx.recv().await {

                                            let req = KvsRequest::Gossip { key, reg };

                                            let mut serialized = Vec::new();
                                            let mut serializer = bincode::Serializer::new(Cursor::new(&mut serialized), options());
                                            Serialize::serialize(&req, &mut serializer).unwrap();

                                            socket
                                                .send(vec![serialized])
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

            let batch_interval_ticker = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
                Duration::from_millis(100),
            ));

            let mut rng = rand::rngs::SmallRng::from_entropy();
            let dist = rand_distr::Zipf::new(1_000_000, dist).unwrap();

            let mut pre_gen_index = 0;
            let pre_gen_random_numbers: Vec<u64> = (0..(128*1024)).map(|_| rng.sample(dist) as u64).collect();

            // Gets feed into the rhs of a lattice join which uses the SetUnion lattice
            // To prevent multiple gets from the same src address being merged they are each tagged with a seq number.
            let get_seq_num = Rc::new(RefCell::new(0usize));

            let mut df = hydroflow_syntax! {

                simulated_put_requests = repeat_fn(2000, move || {
                    let buff = BufferPool::get_from_buffer_pool(&buffer_pool);

                    // Did the original C++ benchmark do anything with the data..?
                    // Can uncomment this to modify the buffers then.
                    //
                    // let mut borrow = buff.borrow_mut().unwrap();

                    // let mut r = rng.sample(dist_uniform) as u64;
                    // for i in 0..8 {
                    //     borrow[i] = (r % 256) as u8;
                    //     r /= 256;
                    // }

                    let key = pre_gen_random_numbers[pre_gen_index % pre_gen_random_numbers.len()];
                    pre_gen_index += 1;

                    (KvsRequest::Put {
                        key,
                        value: buff
                    }, Vec::new())
                });

                merge_puts_and_gossip_requests = merge();

                simulated_put_requests -> merge_puts_and_gossip_requests;
                source_stream(client_to_transducer_rx) -> merge_puts_and_gossip_requests;

                client_input = merge_puts_and_gossip_requests
                    -> demux(|(req, addr): (KvsRequest, Vec<u8>), var_args!(gets, store, broadcast)| {
                        match req {
                            KvsRequest::Put {key, value} => {
                                throughput.fetch_add(1, Ordering::SeqCst);
                                let marker = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() << 5) + (gossip_addr as u128);

                                broadcast.give((key, MyLastWriteWins::new(
                                    Max::new(marker),
                                    Bottom::new(Fake::new(value.clone())),
                                )));

                                store.give((key, MyLastWriteWins::new(
                                    Max::new(marker),
                                    Bottom::new(Fake::new(value)),
                                )));
                            },
                            KvsRequest::Gossip {key, reg} => {
                                store.give((key, reg));
                            },
                            KvsRequest::Get {key} => gets.give((key, addr)),
                            KvsRequest::Delete {key} => {
                                throughput.fetch_add(1, Ordering::SeqCst);
                                let marker = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() << 5) + (gossip_addr as u128);

                                broadcast.give((key, MyLastWriteWins::new(
                                    Max::new(marker),
                                    Bottom::default(),
                                )));

                                store.give((key, MyLastWriteWins::new(
                                    Max::new(marker),
                                    Bottom::default(),
                                )));
                            }
                        }
                    });

                // broadcast out locally generated changes to other nodes.
                client_input[broadcast]
                    -> batch(1000, batch_interval_ticker)
                    -> map(|(key, reg)| (key, Bottom::new(reg)))
                    -> group_by::<'tick>(Bottom::default, Merge::merge) // TODO: Change to reduce syntax.
                    -> filter_map(|(key, opt_reg)| opt_reg.0.map(|reg| (key, reg))) // to filter out bottom types since they carry no useful info.
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: sending to peers: {x:?}", context.current_tick()))
                    -> for_each(|x| { transducer_to_peers_tx.send(x).unwrap(); });

                // join for lookups
                lookup = lattice_join::<'static, 'tick, MyLastWriteWins, MySetUnion>();

                client_input[store]
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: stores-into-lookup: {x:?}", context.current_tick()))
                    -> [0]lookup;

                // Feed gets into the join to make them do the actual matching.
                client_input[gets]
                    -> map(|(key, addr)| {
                        let seq_num = {
                            let mut seq_borrow = get_seq_num.borrow_mut();
                            *seq_borrow += 1;
                            *seq_borrow
                        };

                        (key, SetUnionSingletonSet::new_from((addr, seq_num)))
                    })
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: gets-into-lookup: {x:?}", context.current_tick()))
                    -> [1]lookup;

                // Send get results back to user
                lookup
                    -> map(|(key, (reg, gets))| {
                        gets.0.into_iter().map(move |(dest, _seq_num)| {
                            (KvsResponse::GetResponse { key, reg: reg.clone() }, dest)
                        })
                    })
                    -> flatten()
                    // -> inspect(|x| println!("{gossip_addr}:{:5}: Response to client: {x:?}", context.current_tick()))
                    -> for_each(|x| transducer_to_client_tx.send(x).unwrap());

            };

            let serde_graph = df
                .meta_graph()
                .expect("No graph found, maybe failed to parse.");

            println!("{}", serde_graph.to_mermaid());

            let hydroflow_task = df.run_async();

            futures::join!(inbound_networking_task, outbound_networking_task, hydroflow_task);
        });
    });
}
