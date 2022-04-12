use crate::protocol::{AcceptorResponse, ClientReq, Msg, MsgType, ProposerMsg};
use crate::Opts;
use std::collections::HashMap;

use hydroflow::builder::prelude::*;
use hydroflow::scheduled::handoff::VecHandoff;

// use hydroflow::surface::pull_handoff::{HandoffPullSurface, HandoffPushSurfaceReversed};

use rand::Rng;
fn decide(odds: u8) -> bool {
    let mut rng = rand::thread_rng();
    if rng.gen_range(0..100) > odds {
        return false;
    } else {
        return true;
    };
}

#[derive(Clone, Copy, Debug)]
struct ProposerSlotData {
    val: i32,
    slot: u16,
    ballot: u16,
    p1b_count: u32,
    p2b_count: u32,
}

// fn create_handoff<T>(hf: HydroflowBuilder, name: String) -> (HandoffPushSurfaceReversed<VecHandoff<T>, _>, HandoffPullSurface<VecHandoff<T>>) {
//     let (recv_push, recv_pull) =
//         hf.make_edge::<_, VecHandoff<T>, _>(name);
//     return (recv_push, recv_pull);
// }

pub(crate) async fn run_proposer(opts: Opts) {
    println!("Proposer starting on port {}", opts.port);
    let mut hf = HydroflowBuilder::default();

    // Setup message send/recv ports
    let all_recv = hf.hydroflow.inbound_tcp_vertex_port::<Msg>(opts.port).await;
    let all_recv = hf.wrap_input(all_recv);

    let msg_send = hf.hydroflow.outbound_tcp_vertex::<ProposerMsg>().await;
    let msg_send = hf.wrap_output(msg_send);

    // let (p1a_push, p1a_pull) = hf.make_edge::<_, VecHandoff<ProposerMsg>, _>("p1a_tee");
    // let (p2a_push, p2a_pull) = hf.make_edge::<_, VecHandoff<ProposerMsg>, _>("p2a_tee");

    // hf.add_subgraph_stratified(
    //     "demux",
    //     0,
    //     msg_recv.flatten().pull_to_push().partition(
    //         |m| m.mtype == MsgType::P1A,
    //         hf.start_tee().map(Some).push_to(p1a_push),
    //         hf.start_tee().map(Some).push_to(p2a_push),
    //     )
    // );

    let proposer_id = opts.id;
    let acceptor_count = 3;

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
                move |slots, msg| match msg {
                    Msg::ClientReq(msg) => {
                        let max_slot = slots.keys().max().unwrap_or(&0);
                        // let max_slot = slots.iter().max_by(|a, b| a.0.cmp(b.0)).map(|(k, _)| k);
                        let next_slot = max_slot + 1;

                        let new_slot_entry = ProposerSlotData {
                            val: msg.val,
                            slot: next_slot,
                            ballot: 0,
                            p1b_count: 0,
                            p2b_count: 0,
                        };

                        // Put requested value into hash map
                        slots.insert(next_slot, new_slot_entry);
                        Some(ProposerMsg {
                            addr: opts.addr,
                            slot: new_slot_entry.slot,
                            ballot: new_slot_entry.ballot,
                            pid: proposer_id,
                            val: msg.val,
                            mtype: MsgType::P1A,
                        })
                    }
                    Msg::AcceptorRes(msg) => {
                        let slot_entry = slots.get_mut(&msg.slot).unwrap();
                        let target = acceptor_count / 2 + 1;
                        match msg.mtype {
                            MsgType::P1B => {
                                slot_entry.p1b_count += 1;
                                slot_entry.val = msg.val.unwrap_or(slot_entry.val);

                                if slot_entry.p1b_count == target {
                                    Some(ProposerMsg {
                                        addr: opts.addr,
                                        slot: slot_entry.slot,
                                        ballot: slot_entry.ballot,
                                        pid: proposer_id,
                                        val: slot_entry.val,
                                        mtype: MsgType::P2A,
                                    })
                                } else {
                                    None
                                }
                            }
                            MsgType::P2B => {
                                slot_entry.p2b_count += 1;

                                if slot_entry.p2b_count == target {
                                    println!("yeehaw: {:?}", slot_entry.slot);
                                }
                                None
                            }
                            default => None,
                        }
                    }
                },
            )
            .filter_map(|v| v)
            .map(|msg| {
                println!("Logging {:?}", msg);
                msg
            })
            .pull_to_push()
            .push_to(msg_send),
    );

    let mut hf = hf.build();
    println!("Opening on port {}", opts.port);
    // println!("{}", hf.render_mermaid());

    send_edges.give(Some(Msg::ClientReq(ClientReq { val: 4 })));

    send_edges.flush();
    hf.tick();

    send_edges.give(Some(Msg::ClientReq(ClientReq { val: 7 })));

    send_edges.flush();
    hf.tick();

    // println!("Opening on port {}", opts.port);
    hf.run_async().await.unwrap();
}
