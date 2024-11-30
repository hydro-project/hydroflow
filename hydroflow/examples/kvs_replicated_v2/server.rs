use std::cell::RefCell;
use std::hash::Hasher;
use std::net::SocketAddr;

use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::bind_udp_bytes;
use hydroflow_lang::graph::WriteConfig;
use hydroflow_macro::DemuxEnum;
use lattices::map_union::{MapUnionBTreeMap, MapUnionHashMap, MapUnionSingletonMap};
use lattices::set_union::{SetUnionHashSet, SetUnionSingletonSet};
use lattices::{Atomize, DomPair, Max};
use rustc_hash::FxHasher;

use crate::protocol::{Anna, AnnaValue, Key, NodeId, ServerReqWithSrc, ServerResp, Value};
use crate::Opts;

const HASH_RING: &str = "HASHRING";

// #[derive(Clone, Debug, Hash, Eq, PartialEq, DemuxEnum)]
// enum LookupType {
//     Get(NodeId),
//     Put(NodeId, Value),
//     HashRingGet(NodeId, Key),
//     HashRingPut(NodeId, Key, Value),
// }

fn lookup_in_hash_ring(
    ring: &MapUnionBTreeMap<u64, SetUnionHashSet<NodeId>>,
    hash: u64,
) -> &SetUnionHashSet<NodeId> {
    if let Some((_keyhash, nid)) = ring.as_reveal_ref().range(hash..).next() {
        return &nid;
    }

    if let Some((_keyhash, nid)) = ring.as_reveal_ref().range(..hash).next() {
        return &nid;
    }

    panic!(); // probably the hash ring is empty...
}

pub(crate) async fn run_server(self_node_id: NodeId) {
    let (outbound, inbound, _) = bind_udp_bytes(self_node_id).await;
    println!("Server live! id: {self_node_id:?}");

    let current_version_clock = &*Box::leak(Box::new(RefCell::new(0usize)));

    let mut hf: Hydroflow = hydroflow_syntax! {
        network_recv = source_stream_serde(inbound)
            -> map(Result::unwrap)
            // -> inspect(|(msg, addr)| println!("Message received {:?} from {:?}", msg, addr))
            -> map(|(msg, addr)| ServerReqWithSrc::from_server_req(msg, addr))
            -> demux_enum::<ServerReqWithSrc>();

        network_recv[RemoveNode]
            -> null();

        // Request to add a node to the hash ring.
        network_recv[AddNode]
            -> map(|(_src, node_id)| {
                let mut current_version = current_version_clock.borrow_mut();
                *current_version += 1;
                let new_version = *current_version;

                let mut hasher = FxHasher::default();
                std::hash::Hash::hash(&node_id, &mut hasher);
                let hash = hasher.finish();

                println!("Adding Node: {} with hash: {}", node_id, hash);

                // TODO: This is not correct, it overwrites the whole hash ring when a new node is added
                // it needs to combine with the existing hash ring, but that would require first getting the hash ring
                // which would require another trip through the join state.
                MapUnionSingletonMap::new_from((
                    HASH_RING.to_owned(),
                    DomPair::new(
                        MapUnionSingletonMap::new_from((self_node_id, Max::new(new_version))),
                        AnnaValue::HashRing(MapUnionBTreeMap::new_from([(hash, SetUnionHashSet::new_from([node_id]))])),
                    )),
                )
            })
            -> _upcast(Some(Delta))
            -> lattice_updates;

        lattice_updates = union()
            -> persist()
            -> lattice_fold(Anna::default) // This is where all the anna state lives, hash ring and writes. This should be gossiped.
            -> identity::<Anna>()
            // -> inspect(|x| println!("{}, after_lattice_fold. x: {x:?}", context.current_tick()))
            -> flat_map(|x: Anna| {
                // should be able to atomize this thing without it being so manual.
                // Not sure what atomizing a dompair means tho.
                x.into_reveal().into_iter().map(|(k, v)| {
                    (k.clone(), v.into_reveal().1)
                })
            }) // probably this is where the gossping should be hooked in, everything after this is internal representation for querying.
            -> demux(|(k, v): (String, AnnaValue), var_args!(hash_ring_updates, puts)| {
                eprintln!("k: {k}, v: {v:?}");
                if k.as_str() == HASH_RING {
                    hash_ring_updates.give(v);
                } else {
                    puts.give((k, v));
                }
            });

        lattice_updates[hash_ring_updates]
            // -> inspect(|x| println!("{}, lattice_updates[hash_ring_updates] output. x: {x:?}", context.current_tick()))
            -> [0]hash_ring_lookup;

        network_recv[ClientGet]
            -> map(|(src, key): (SocketAddr, String)| ServerReqWithSrc::ClientGet{src, key})
            -> hash_ring_lookup_union;

        network_recv[ClientPut]
            // -> inspect(|x| println!("network_recv[ClientPut] output. x: {x:?}"))
            -> map(|(src, key, value): (SocketAddr, String, String)| ServerReqWithSrc::ClientPut{src, key, value})
            -> hash_ring_lookup_union;

        hash_ring_lookup_union = union()
            -> [1]hash_ring_lookup;

        hash_ring_lookup = cross_join_multiset()
            // -> inspect(|(x, y)| println!("hash_ring_lookup output. x: {x:?}, y: {y:?}"))
            -> filter_map(|(hash_ring, req)| {
                let key = match &req {
                    ServerReqWithSrc::ClientPut { src, key, value } => key,
                    ServerReqWithSrc::ClientGet { src, key } => key,
                    ServerReqWithSrc::AddNode { src, node_id } => panic!(),
                    ServerReqWithSrc::RemoveNode { src, node_id } => panic!(),
                };

                let mut hasher = FxHasher::default();
                std::hash::Hash::hash(&key, &mut hasher);
                let hash = hasher.finish();

                let AnnaValue::HashRing(hash_ring) = hash_ring else {
                    panic!();
                };

                if lookup_in_hash_ring(&hash_ring, hash).as_reveal_ref().contains(&self_node_id) {
                    Some(req)
                } else {
                    eprintln!("key: {key}, not owned by node: {self_node_id}, keyhash: {hash}, hash_ring: {hash_ring:?}");
                    None
                }
            })
            -> defer_tick() // TODO: to mitigate some persist() -> fold issue causing the graph to tick endlessly even with no data flowing.
            -> demux_enum::<ServerReqWithSrc>();

        hash_ring_lookup[ClientGet]
            -> map(|(src, k)| (k, src))
            // -> inspect(|(key, src)| println!("hash_ring_lookup[ClientGet]. key: {key}, src: {src}"))
            -> [1]puts_and_gets;
        hash_ring_lookup[ClientPut]
            -> map(|(src, k, v)| {
                let mut current_version = current_version_clock.borrow_mut();
                *current_version += 1;
                let new_version = *current_version;

                MapUnionSingletonMap::new_from((
                    k,
                    DomPair::new(
                        MapUnionSingletonMap::new_from((self_node_id, Max::new(new_version))),
                        AnnaValue::Value(SetUnionHashSet::new_from([v])),
                    )),
                )
            })
            -> _upcast(Some(Delta))
            -> lattice_updates;

        hash_ring_lookup[AddNode]
            -> inspect(|_| panic!())
            -> null();

        hash_ring_lookup[RemoveNode]
            -> inspect(|_| panic!())
            -> null();

        lattice_updates[puts]
            -> map(|(key, value)| {
                let AnnaValue::Value(value) = value else {
                    panic!();
                };

                (key, value)
            })
            -> [0]puts_and_gets;

        puts_and_gets = import!("right_outer_join.hf")
            -> inspect(|(key, (x, y))| println!("puts_and_gets output. key: {key}, x: {x:?}, y: {y}"))
            -> map(|(key, (value, src))| (ServerResp::ServerResponse{ key, value: value.map(|x| x.into_reveal()) }, src))
            -> dest_sink_serde(outbound);




            // -> [0]puts_and_gets;

        // demux(|(k, v): (String, AnnaValue), var_args!(hash_ring_updates, puts)| {

        // });

        // hash_ring_join = join_multiset()
        //     -> null();

        // upgrade request to lattice delta
        // For a put, first we have to figure out if we are responsible for this key or not.


        // into_lookup = union()
        //     -> [1]puts_and_gets;

        // network_recv[ClientPut]
        //     -> persist() // Although lattice_fold complains about it, we need this here.
        //     -> map(|(_src, k, v): (SocketAddr, String, String)| {
        //         let mut current_version = current_version_clock.borrow_mut();
        //         *current_version += 1;
        //         let new_version = *current_version;

        //         MapUnionSingletonMap::new_from((
        //             k,
        //             DomPair::new(
        //                 MapUnionSingletonMap::new_from((node_id, Max::new(new_version))),
        //                 AnnaValue::Value(SetUnionHashSet::new_from([v])),
        //             )),
        //         )
        //     }) -> store_writes;

        // puts_and_gets = import!("right_outer_join.hf")
        //     -> filter_map(|(key, (ring, action))| {

        //         match action {
        //             LookupType::HashRingGet(src, key) => {
        //                 let mut hasher = FxHasher::default();
        //                 std::hash::Hash::hash(&key, &mut hasher);
        //                 let hash = hasher.finish();

        //                 eprintln!("key: {} -> {}", key, hash);

        //                 match ring {
        //                     Some(AnnaValue::HashRing(ring)) => {
        //                         if lookup_in_hash_ring(&ring, hash).as_reveal_ref().contains(&self_node_id) {
        //                             Some((key.clone(), LookupType::Get(src)))
        //                         } else {
        //                             eprintln!("key: {key}, not owned by node: {self_node_id}");
        //                             None
        //                         }
        //                     },
        //                     _ => panic!(),
        //                 }
        //             },
        //             LookupType::HashRingPut(src, key, value) => {
        //                 let mut hasher = FxHasher::default();
        //                 std::hash::Hash::hash(&key, &mut hasher);
        //                 let hash = hasher.finish();

        //                 eprintln!("key: {} -> {}", key, hash);

        //                 match ring {
        //                     Some(AnnaValue::HashRing(ring)) => {
        //                         if lookup_in_hash_ring(&ring, hash).as_reveal_ref().contains(&self_node_id) {
        //                             Some((key.clone(), LookupType::Put(src, value)))
        //                         } else {
        //                             eprintln!("key: {key}, not owned by node: {self_node_id}");
        //                             None
        //                         }
        //                     },
        //                     _ => panic!(),
        //                 }
        //             },
        //             LookupType::Get(src) => {
        //                 println!("get. key: {key}");
        //                 None
        //             },
        //             LookupType::Put(src, v) => {
        //                 println!("put. key: {key}, value: {v}");
        //                 None
        //             },
        //         }
        //     })
        //     -> inspect(|x| eprintln!("{:?}", x))
        //     -> into_lookup;

        // // Join PUTs and GETs by key
        // writes = union() -> tee();
        // writes -> map(|(key, value, _addr)| (key, value)) -> writes_store;
        // writes_store = persist() -> tee();
        // writes_store -> [0]lookup;
        // gets -> [1]lookup;
        // lookup = join();

        // network_send = union() -> dest_sink_serde(outbound);

        // // Send GET responses back to the client address.
        // lookup[1]
        //     -> inspect(|tup| println!("Found a match: {:?}", tup))
        //     -> map(|(key, (value, client_addr))| (KvsMessage::ServerResponse { key, value }, client_addr))
        //     -> network_send;

        // // Join as a peer if peer_server is set.
        // source_iter_delta(peer_server) -> map(|peer_addr| (KvsMessage::PeerJoin, peer_addr)) -> network_send;

        // // Peers: When a new peer joins, send them all data.
        // writes_store -> [0]peer_join;
        // peers -> [1]peer_join;
        // peer_join = cross_join()
        //     -> map(|((key, value), peer_addr)| (KvsMessage::PeerGossip { key, value }, peer_addr))
        //     -> network_send;

        // // Outbound gossip. Send updates to peers.
        // peers -> peer_store;
        // source_iter_delta(peer_server) -> peer_store;
        // peer_store = union() -> persist();
        // writes -> [0]outbound_gossip;
        // peer_store -> [1]outbound_gossip;
        // outbound_gossip = cross_join()
        //     // Don't send gossip back to the sender.
        //     -> filter(|((_key, _value, writer_addr), peer_addr)| writer_addr != peer_addr)
        //     -> map(|((key, value, _writer_addr), peer_addr)| (KvsMessage::PeerGossip { key, value }, peer_addr))
        //     -> network_send;
    };

    // let serde_graph = hf
    //     .meta_graph()
    //     .expect("No graph found, maybe failed to parse.");
    // serde_graph
    //     .open_graph(
    //         hydroflow_lang::graph::WriteGraphType::Mermaid,
    //         Some(WriteConfig {
    //             op_short_text: true,
    //             ..Default::default()
    //         }),
    //     )
    //     .unwrap();

    hf.run_async().await.unwrap();
}
