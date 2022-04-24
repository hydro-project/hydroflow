use crate::protocol::{ClientReq, Msg, MsgType, ProposerReq};
use crate::Opts;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
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

    let use_proxy = opts.use_proxy;

    // let (send_edges, recv_edges) = hf.add_channel_input::<_, _, VecHandoff<usize>>("edge input");
    let (send_edges, recv_edges) = hf.add_channel_input::<_, _, VecHandoff<Msg>>("start input");

    hf.add_subgraph_stratified(
        "Main processing",
        0,
        all_recv
            .chain(recv_edges) // TODO: temporary, for testing
            .flatten()
            .map_scan(0 as i32, move |slot_counter, msg| {
                let resp = match msg {
                    Msg::ClientReq(msg) => {
                        *slot_counter += 1;
                        let hashed = waste_time(hash_u16((*slot_counter) as u16));
                        //let hashed = hash_u16(*max_slot);
                        //slots.insert(
                        //max_slot + 1,
                        //ProposerSlotData {
                        //val: 0,
                        //slot: 0,
                        //ballot: 0,
                        //p1b_count: 0,
                        //p2b_count: 0,
                        //hash: hashed,
                        //},
                        //);

                        Some(Msg::ProposerReq(ProposerReq {
                            addr: opts.addr.clone(),
                            slot: *slot_counter,
                            ballot: 0,
                            pid: 0,
                            val: msg.val,
                            mtype: MsgType::P1A,
                        }))
                    }
                    default => None,
                };

                // if using proxy leaders, send to proxy leader
                let mut vec = VecDeque::<(String, Msg)>::new();
                if use_proxy {
                    let addr = opts.proxy_addrs
                        [((*slot_counter) as usize) % opts.proxy_addrs.len()]
                    .clone();
                    vec.push_back(((*addr).to_string(), resp.clone().unwrap()));
                } else {
                    for addr in opts.acceptor_addrs.clone().into_iter() {
                        vec.push_back((addr, resp.clone().unwrap()));
                    }
                }
                vec
            })
            // .filter_map(|v| v)
            .pull_to_push()
            // .map(Some)
            .push_to(msg_send),
    );

    let mut hf = hf.build();
    println!("Opening on port {}", opts.port);
    // println!("{}", hf.render_mermaid());

    let mut total_counter = 0;
    let mut counter = 0;
    let mut rng = rand::thread_rng();
    let message_interval = Duration::from_micros(0); //40);
    let mut start = SystemTime::now();
    let mut prev_iter_time = start;
    let mut warmup = true;

    let mut total_flush_time = 0.0;

    while total_counter < 300000 + 100 {
        // wait until message_interval has passed
        // let now = SystemTime::now();
        // let elapsed = now.duration_since(prev_iter_time).unwrap();
        // if elapsed < message_interval {
        //     std::thread::sleep(message_interval - elapsed);
        // }
        // prev_iter_time = now; //SystemTime::now();

        // send a message through the channel
        let now = SystemTime::now();
        send_edges.give(Some(Msg::ClientReq(ClientReq { val: rng.gen() })));
        send_edges.flush();
        hf.tick();
        let tick_time = now.elapsed().unwrap();
        total_flush_time += tick_time.as_secs() as f64 * 1000.0 + tick_time.subsec_nanos() as f64 / 1_000_000.0;

        counter += 1;
        total_counter += 1;
        if counter % 10000 == 0 {
            let elapsed = start.elapsed().unwrap();
            let elapsed_ms = elapsed.as_secs() as f64 * 1000.0 + elapsed.subsec_nanos() as f64 / 1_000_000.0;
            println!(
                "Counter {}, Elapsed {}, Throughput {}, Avg flush time {}, Avg time per iter {}",
                total_counter,
                elapsed_ms,
                counter as f64 / elapsed_ms as f64 * 1000.0,
                total_flush_time as f64 / total_counter as f64,
                elapsed_ms as f64 / counter as f64,
            );
            if warmup {
                start = SystemTime::now();
                counter = 0;
                warmup = false;
            }
        }
    }

    // println!("Opening on port {}", opts.port);
    hf.run_async().await.unwrap();

    // // wait for 5 seconds
    // std::thread::sleep(Duration::from_secs(5));

    println!("Closing proposer on port {}", opts.port);
}
