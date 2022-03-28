use crate::protocol::{CoordMsg, MsgType, SubordResponse};
use crate::Opts;
use hydroflow::builder::prelude::*;
use hydroflow::scheduled::handoff::VecHandoff;

use rand::Rng;
fn decide(odds: u8) -> bool {
    let mut rng = rand::thread_rng();
    if rng.gen_range(0..100) > odds {
        return false;
    } else {
        return true;
    };
}

pub(crate) async fn run_subordinate(opts: Opts, coordinator: String) {
    let mut hf = HydroflowBuilder::default();

    // setup message send/recv ports
    let msg_recv = hf
        .hydroflow
        .inbound_tcp_vertex_port::<CoordMsg>(opts.port)
        .await;
    let msg_recv = hf.wrap_input(msg_recv);
    let msg_send = hf.hydroflow.outbound_tcp_vertex::<SubordResponse>().await;
    let msg_send = hf.wrap_output(msg_send);

    // Demultiplex received messages into phase1, phase2, and end-of-phase2 flows
    let (p1_recv_push, p1_recv_pull) = hf.make_edge::<_, VecHandoff<CoordMsg>, _>("p1 recv");
    let (p2_recv_push, p2_recv_pull) = hf.make_edge::<_, VecHandoff<CoordMsg>, _>("p2 recv");
    let (p2_end_recv_push, p2_end_recv_pull) =
        hf.make_edge::<_, VecHandoff<CoordMsg>, _>("p2 end recv");
    hf.add_subgraph(
        "demux",
        msg_recv.flatten().pull_to_push().partition(
            |m| m.mtype == MsgType::Prepare,
            hf.start_tee().map(Some).push_to(p1_recv_push),
            hf.start_tee().partition(
                |m| m.mtype == MsgType::End,
                hf.start_tee().map(Some).push_to(p2_end_recv_push),
                hf.start_tee().map(Some).push_to(p2_recv_push),
            ),
        ),
    );

    // Multiplex messages to send
    let (p1_send_push, p1_send_pull) =
        hf.make_edge::<_, VecHandoff<(String, SubordResponse)>, _>("p1 send");
    let (p2_resp_push, p2_resp_pull) =
        hf.make_edge::<_, VecHandoff<(String, SubordResponse)>, _>("p2 respond");
    hf.add_subgraph(
        "mux",
        p1_send_pull
            .chain(p2_resp_pull)
            .pull_to_push()
            .push_to(msg_send),
    );

    // set up addressing. There has to be a nicer way to reuse strings!
    let coord_addr = coordinator.clone();
    let coord_addr_2 = coordinator.clone();
    let my_addr = format!("{}:{}", opts.addr, opts.port);
    let my_addr_2 = format!("{}:{}", opts.addr, opts.port);
    let my_addr_3 = format!("{}:{}", opts.addr, opts.port);

    // Phase one (Prepare) request handling
    // We flip a coin to decide if we will commit or abort
    // and then send a phase-1 response to the coordinator
    hf.add_subgraph(
        "p1 request handler",
        p1_recv_pull
            .flatten()
            .map(move |msg| {
                println!("Xid {:?}: got a {:?}", msg.xid, msg.mtype);
                let ret = SubordResponse {
                    xid: msg.xid,
                    mid: msg.mid + 1,
                    addr: my_addr.clone(),
                    mtype: match msg.mtype {
                        MsgType::Prepare if decide(67) => MsgType::Commit,
                        MsgType::Prepare => MsgType::Abort,
                        _ => MsgType::Err,
                    },
                };
                println!("Xid {:?}: returned a {:?}", ret.xid, ret.mtype);
                Some((coord_addr.clone(), ret))
            })
            .pull_to_push()
            .push_to(p1_send_push),
    );

    // Phase two (Commit/Abort) request handling
    // We should log this, and then we respond with an AckP2
    hf.add_subgraph(
        "p2 command handler",
        p2_recv_pull
            .flatten()
            .map(move |msg| {
                println!("Xid {:?}: got a {:?}", msg.xid, msg.mtype);
                let ret = SubordResponse {
                    xid: msg.xid,
                    mid: msg.mid + 1,
                    addr: my_addr_2.clone(),
                    mtype: match msg.mtype {
                        MsgType::Abort | MsgType::Commit => MsgType::AckP2,
                        _ => MsgType::Err,
                    },
                };
                println!("Xid {:?}: returned a {:?}", ret.xid, ret.mtype);
                Some((coord_addr_2.clone(), ret))
            })
            .pull_to_push()
            .push_to(p2_resp_push),
    );

    // Phase three (End) request handling
    // We should log this, and then we respond with Ended (allows coordinator to GC the transaction)
    hf.add_subgraph(
        "p2 end handler",
        p2_end_recv_pull
            .flatten()
            .map(move |msg| {
                println!("Xid {:?}: got a {:?}", msg.xid, msg.mtype);
                let ret = SubordResponse {
                    xid: msg.xid,
                    mid: msg.mid + 1,
                    addr: my_addr_3.clone(),
                    mtype: match msg.mtype {
                        MsgType::End => MsgType::Ended,
                        _ => MsgType::Err,
                    },
                };
                println!("Xid {:?}: returned a {:?}", ret.xid, ret.mtype);
                Some(ret)
            })
            .pull_to_push()
            .for_each(|m| println!("Logging final message {:?}", m.unwrap())),
    );

    let mut hf = hf.build();
    println!("Opening on port {}", opts.port);
    if opts.mermaid {
        println!("{}", hf.generate_mermaid());
    }
    if opts.dot {
        println!("{}", hf.generate_dot());
    }
    hf.run_async().await.unwrap();
}
