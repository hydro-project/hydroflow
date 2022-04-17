use crate::protocol::{ClientReq, Msg, MsgType, ProposerReq};
use crate::Opts;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::time::{Duration, SystemTime};

use hydroflow::builder::prelude::*;
use hydroflow::scheduled::handoff::VecHandoff;

// use hydroflow::surface::pull_handoff::{HandoffPullSurface, HandoffPushSurfaceReversed};

#[derive(Clone, Copy, Debug)]
struct ProposerSlotData {
    val: i32,
    slot: u16,
    ballot: u16,
    p1b_count: u32,
    p2b_count: u32,
    hash: u64,
}

fn hash_u16(x: u16) -> u64 {
    let mut s = DefaultHasher::new();
    x.hash(&mut s);
    s.finish()
}

fn calculate_hash(val: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    val.hash(&mut hasher);
    hasher.finish()
}

fn waste_time(val: u64) -> u64 {
    let mut current_val = calculate_hash(val);
    for _ in 0..1_000 {
        let new_val = calculate_hash(current_val);
        current_val = new_val;
    }
    current_val
}

pub(crate) async fn run_proposer(opts: Opts) {
    println!("Proposer starting on port {}", opts.port);
    let mut hf = HydroflowBuilder::default();

    // Setup message send/recv ports
    let all_recv = hf.hydroflow.inbound_tcp_vertex_port::<Msg>(opts.port).await;
    let all_recv = hf.wrap_input(all_recv);

    let msg_send = hf.hydroflow.outbound_tcp_vertex::<Msg>().await;
    let msg_send = hf.wrap_output(msg_send);

    // let (send_edges, recv_edges) = hf.add_channel_input::<_, _, VecHandoff<usize>>("edge input");
    let (send_edges, recv_edges) = hf.add_channel_input::<_, _, VecHandoff<Msg>>("start input");

    hf.add_subgraph_stratified(
        "Main processing",
        0,
        all_recv
            .chain(recv_edges) // TODO: temporary, for testing
            .flatten()
            .map_scan(
                HashMap::<u16, ProposerSlotData>::new(),
                move |slots, msg| {
                    let resp = match msg {
                        Msg::ClientReq(msg) => {
                            let max_slot = slots.keys().max().unwrap_or(&0);
                            let hashed = waste_time(hash_u16(*max_slot));
                            // let hashed = hash_u16(*max_slot);
                            // slots.insert(
                            //     max_slot + 1,
                            //     ProposerSlotData {
                            //         val: 0,
                            //         slot: 0,
                            //         ballot: 0,
                            //         p1b_count: 0,
                            //         p2b_count: 0,
                            //         hash: hashed,
                            //     },
                            // );

                            Some(Msg::ProposerReq(ProposerReq {
                                addr: opts.addr.clone(),
                                slot: 0,
                                ballot: 0,
                                pid: 0,
                                val: msg.val,
                                mtype: MsgType::P1A,
                            }))
                        }
                        default => None,
                    };

                    let mut vec = VecDeque::<(String, Msg)>::new();
                    for addr in opts.acceptor_addrs.clone().into_iter() {
                        vec.push_back((addr, resp.clone().unwrap()));
                    }
                    vec
                },
            )
            // .filter_map(|v| v)
            .pull_to_push()
            // .map(Some)
            .push_to(msg_send),
    );

    let mut hf = hf.build();
    println!("Opening on port {}", opts.port);
    // println!("{}", hf.render_mermaid());

    let mut counter = 0;
    let mut rng = rand::thread_rng();
    let start = SystemTime::now();

    while counter < 100000 {
        send_edges.give(Some(Msg::ClientReq(ClientReq { val: rng.gen() })));
        send_edges.flush();
        hf.tick();
        counter += 1;
        if counter % 1000 == 0 {
            let elapsed = start.elapsed().unwrap();
            let elapsed_ms = elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1_000_000;
            println!(
                "Counter {}, Elapsed {}, Throughput {}",
                counter,
                elapsed_ms,
                counter as f64 / elapsed_ms as f64 * 1000.0
            );
        }
    }

    // println!("Opening on port {}", opts.port);
    hf.run_async().await.unwrap();
}
