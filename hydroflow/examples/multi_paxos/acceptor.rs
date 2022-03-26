use std::collections::HashMap;
use crate::protocol::{ProposerMsg, MsgType, AcceptorResponse};
use hydroflow::builder::prelude::*;
use hydroflow::scheduled::handoff::{VecHandoff};
use crate::Opts;
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

    let (p1a_push, p1a_pull) = hf.make_edge::<_, VecHandoff<ProposerMsg>, _>("p1a_tee");
    let (p2a_push, p2a_pull) = hf.make_edge::<_, VecHandoff<ProposerMsg>, _>("p2a_tee");

    hf.add_subgraph_stratified(
        "demux",
        0,
        msg_recv.flatten().pull_to_push().partition(
            |m| m.mtype == MsgType::P1A,
            hf.start_tee().map(Some).push_to(p1a_push),
            hf.start_tee().map(Some).push_to(p2a_push),
        )
    );

    // let (send_edges, recv_edges) = hf.add_channel_input::<_, _, VecHandoff<usize>>("edge input");
    let (send_edges, recv_edges) =
        hf.add_channel_input::<_, _, VecHandoff<u16>>("start input");


    hf.add_subgraph_stratified("name", 0,
        recv_edges 
            .flatten()
            .map_scan(
                HashMap::<u16, u16>::new(),
                |counts, msg| {
                    let v = counts.entry(msg).or_insert(0);
                    print!("Map: {:?} --> {:?}", msg, v);
                    *v += 1;
                    *v
                },
            )
            .map(|msg| {
                println!("Logging {:?}", msg);
                msg
            })
            .pull_to_push()
            .for_each(|_| {})
    );
    
    let mut hf = hf.build();


    send_edges.give(Some(5));
    send_edges.give(Some(7));
    send_edges.give(Some(5));
    send_edges.flush();
    hf.tick();

    send_edges.give(Some(5));
    send_edges.flush();
    hf.tick();
    println!("HEHEHE");

    // println!("Opening on port {}", opts.port);
    // hf.run_async().await.unwrap();
}
