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

pub(crate) async fn run_proxy_leader(opts: Opts) {
    let mut hf = HydroflowBuilder::default();

    // Setup message send/recv ports
    let all_recv = hf.hydroflow.inbound_tcp_vertex_port::<Msg>(opts.port).await;
    let all_recv = hf.wrap_input(all_recv);

    let msg_send = hf.hydroflow.outbound_tcp_vertex::<Msg>().await;
    let msg_send = hf.wrap_output(msg_send);

    hf.add_subgraph_stratified(
        "Broadcast",
        0,
        all_recv
            .inspect(|v| println!("{}", v.len()))
            .flatten()
            .map(move |msg| {
                let mut vec = VecDeque::<(String, Msg)>::new();
                for addr in opts.acceptor_addrs.clone().into_iter() {
                    vec.push_back((addr, msg.clone()));
                }
                vec
            })
            .pull_to_push()
            .push_to(msg_send),
    );

    let mut hf = hf.build();
    println!("Proxy leader starting on port {}", opts.port);
    hf.run_async().await.unwrap();
}

