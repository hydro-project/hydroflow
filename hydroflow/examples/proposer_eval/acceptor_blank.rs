use crate::protocol::{AcceptorResponse, Msg, MsgType};
use hydroflow::builder::prelude::*;
use std::time::{Duration, SystemTime};

pub(crate) async fn run_acceptor(port: u16) {
    let mut hf = HydroflowBuilder::default();

    // // Setup message send/recv ports
    let msg_recv = hf.hydroflow.inbound_tcp_vertex_port::<Msg>(port).await;
    let msg_recv = hf.wrap_input(msg_recv);

    let mut start_time = SystemTime::now();
    let mut total_counter = 0;

    hf.add_subgraph_stratified(
        "Main processing",
        0,
        msg_recv.flatten()
        .map_scan(0 as i32, move |recv_counter, msg| {
            *recv_counter += 1;
            total_counter += 1;
            if *recv_counter % 10000 == 0 {
                let elapsed = start_time.elapsed().unwrap();
                let elapsed_ms = elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1_000_000;
                println!("Acceptor Counter {}, Elapsed {}, Throughput {}", total_counter, elapsed_ms, *recv_counter as f64 / elapsed_ms as f64 * 1000.0);
                start_time = SystemTime::now();
                *recv_counter = 0;
            }
        }).pull_to_push().for_each(|_| {})
    );

    let mut hf = hf.build();
    println!("Opening on port {}", port);
    hf.run_async().await.unwrap();
}
