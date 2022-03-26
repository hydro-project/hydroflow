use std::collections::HashMap;
use crate::protocol::{ProposerMsg, MsgType, AcceptorResponse};
use hydroflow::builder::prelude::*;
use hydroflow::scheduled::handoff::VecHandoff;
use crate::Opts;
use std::cmp::Ordering;

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

#[derive(PartialEq, Clone, Copy, Debug)]
struct SlotData {
    slot: u16,
    ballot: u16,
    pid: u16,
    val: Option<i32>,
}

impl PartialOrd for SlotData {
    fn partial_cmp(&self, s2: &Self) -> Option<Ordering> {
        if self.ballot > s2.ballot {
            return Some(Ordering::Greater);
        } else if self.ballot < s2.ballot {
            return Some(Ordering::Less);
        } else if self.pid > s2.pid {
            return Some(Ordering::Greater);
        } else if self.pid < s2.pid {
            return Some(Ordering::Less);
        }
        return Some(Ordering::Equal);
    }
}

// fn create_handoff<T>(hf: HydroflowBuilder, name: String) -> (HandoffPushSurfaceReversed<VecHandoff<T>, _>, HandoffPullSurface<VecHandoff<T>>) {
//     let (recv_push, recv_pull) =
//         hf.make_edge::<_, VecHandoff<T>, _>(name);
//     return (recv_push, recv_pull);
// }

pub(crate) async fn run_acceptor(opts: Opts) {
    println!("Acceptor starting on port {}", opts.port);
    let mut hf = HydroflowBuilder::default();

    // Setup message send/recv ports
    let msg_recv = hf
        .hydroflow
        .inbound_tcp_vertex_port::<ProposerMsg>(opts.port)
        .await;
    let msg_recv = hf.wrap_input(msg_recv);
    // let msg_send = hf.hydroflow.outbound_tcp_vertex::<AcceptorResponse>().await;
    // let msg_send = hf.wrap_output(msg_send);

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

    // let (send_edges, recv_edges) = hf.add_channel_input::<_, _, VecHandoff<usize>>("edge input");
    let (send_edges, recv_edges) =
        hf.add_channel_input::<_, _, VecHandoff<ProposerMsg>>("start input");


    hf.add_subgraph_stratified("Main processing", 0,
        recv_edges 
            .flatten()
            // .map(|msg| {
            //     ProposerMsg {
            //         slot: msg.0,
            //         ballot: msg.1,
            //         pid: 3,
            //         val: 3,
            //         mtype: MsgType::P1A,
            //         mtype: msg.4,
            //     }
            // })
            .map_scan(
                HashMap::<u16, SlotData>::new(),
                |slots, msg| {
                    let mut win = false;
                    let mut s = SlotData {
                        slot: msg.slot,
                        ballot: msg.ballot,
                        pid: msg.pid,
                        val: None,
                    };
                    let v = slots.entry(msg.slot).or_insert(s);

                    // Phase 1
                    let resp = match msg.mtype {
                        MsgType::P1A => {
                            if &s > v {
                                s.val = v.val;
                                slots.insert(msg.slot, s);
                                Some(AcceptorResponse {
                                    slot: s.slot,
                                    ballot: s.ballot,
                                    pid: s.pid,
                                    accepted_val: s.val,
                                    accepted_ballot: if s.val.is_none() {None} else { Some(s.ballot)},
                                    win: true,
                                    val: None,
                                    mtype: MsgType::P1B,
                                })
                            }
                            else {
                                // don't send anything for now
                                None
                            }
                        },

                        MsgType::P2A => {
                            s.val = Some(msg.val);
                            if &s >= v {
                                if v.val.is_some() {
                                    assert!(s.val == v.val);
                                }
                                v.val = s.val;
                                println!("P2A Accept");
                                slots.insert(msg.slot, s);
                                Some(AcceptorResponse {
                                    slot: s.slot,
                                    ballot: s.ballot,
                                    pid: s.pid,
                                    accepted_val: s.val,
                                    accepted_ballot: Some(s.ballot),
                                    win: true,
                                    val: s.val,
                                    mtype: MsgType::P2B,
                                })
                            }
                            else {
                                // don't send anything for now
                                None
                            }
                        },
                        _ => {
                            println!("Acceptors only receive P1A and P2A messages");
                            None
                        },
                    };

                    println!("{:?}", slots);
                    // print!("Map: {:?} --> {:?}", msg, v);
                    resp.clone()
                },
            ).filter_map(|v| {
                v
            })
            .map(|msg| {
                println!("Logging {:?}", msg);
                msg
            })
            .pull_to_push()
            .for_each(|_| {})
    );
    
    let mut hf = hf.build();
    println!("Opening on port {}", opts.port);
    // println!("{}", hf.render_mermaid());

    send_edges.give(Some(ProposerMsg{
        slot: 0,
        ballot: 0,
        pid: 0,
        val: 4,
        mtype: MsgType::P1A,
    }));

    send_edges.flush();
    hf.tick();

    send_edges.give(Some(ProposerMsg{
        slot: 0,
        ballot: 1,
        pid: 0,
        val: 7,
        mtype: MsgType::P1A,
    }));

    send_edges.give(Some(ProposerMsg{
        slot: 0,
        ballot: 1,
        pid: 1,
        val: 20,
        mtype: MsgType::P1A,
    }));

    send_edges.give(Some(ProposerMsg{
        slot: 0,
        ballot: 0,
        pid: 0,
        val: 4,
        mtype: MsgType::P2A,
    }));
    send_edges.flush();
    hf.tick();

    send_edges.give(Some(ProposerMsg{
        slot: 0,
        ballot: 1,
        pid: 1,
        val: 20,
        mtype: MsgType::P2A,
    }));

    send_edges.give(Some(ProposerMsg{
        slot: 0,
        ballot: 1,
        pid: 0,
        val: 16,
        mtype: MsgType::P2A,
    }));

    // send_edges.give(Some(ProposerMsg{
    //     slot: 0,
    //     ballot: 1,
    //     pid: 0,
    //     val: 7,
    //     mtype: MsgType::P1A,
    // }));

    // send_edges.give(Some(ProposerMsg{
    //     slot: 0,
    //     ballot: 1,
    //     pid: 1,
    //     val: 20,
    //     mtype: MsgType::P1A,
    // }));

    // send_edges.give(Some(ProposerMsg{
    //     slot: 0,
    //     ballot: 0,
    //     pid: 0,
    //     val: 0,
    //     mtype: MsgType::P1A,
    // }));

    send_edges.flush();

    // println!("Opening on port {}", opts.port);
    hf.run_async().await.unwrap();
}
