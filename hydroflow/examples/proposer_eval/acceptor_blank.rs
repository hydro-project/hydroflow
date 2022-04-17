use crate::protocol::{AcceptorResponse, Msg, MsgType};
use crate::Opts;
use hydroflow::builder::prelude::*;
use hydroflow::scheduled::handoff::VecHandoff;
use std::cmp::Ordering;
use std::collections::HashMap;

pub(crate) async fn run_acceptor(port: u16) {
    println!("here");
    let mut hf = HydroflowBuilder::default();

    // // Setup message send/recv ports
    let msg_recv = hf.hydroflow.inbound_tcp_vertex_port::<Msg>(port).await;
    let msg_recv = hf.wrap_input(msg_recv);

    hf.add_subgraph_stratified(
        "Main processing",
        0,
        msg_recv.flatten().pull_to_push().for_each(|_| {}),
    );

    let mut hf = hf.build();
    println!("Opening on port {}", port);
    hf.run_async().await.unwrap();
}
