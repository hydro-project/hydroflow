use crate::protocol::{AcceptorResponse, Msg, MsgType, ThroughputMeasurement};
use hydroflow::builder::prelude::*;
use std::time::{Duration, SystemTime};

pub(crate) async fn run_acceptor(port: u16) {
    let mut hf = HydroflowBuilder::default();

    // // Setup message send/recv ports
    let msg_recv = hf.hydroflow.inbound_tcp_vertex_port::<Msg>(port).await;
    let msg_recv = hf.wrap_input(msg_recv);

    let mut start_time = SystemTime::now();
    let mut total_counter = 0;

    let mut throughput_vec = Vec::new();

    hf.add_subgraph_stratified(
        "Main processing",
        0,
        msg_recv.flatten()
        .map_scan(0 as i32, move |recv_counter, msg| {
            println!("Message: {:?}", msg);
            match msg {
                Msg::ProposerReq(msg) => {
                    *recv_counter += 1;
                    total_counter += 1;
                    let msg_slot_counter = msg.slot as i32;
                    
                    println!("Acceptor received");
                    // assert!(msg_slot_counter == *recv_counter);
                    if *recv_counter % 10000 == 0 {
                        let elapsed = start_time.elapsed().unwrap();
                        let elapsed_ms = elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1_000_000;
                        println!("Acceptor Counter {}, Elapsed {}, Throughput {}, Message Slot Counter {}", total_counter, elapsed_ms, *recv_counter as f64 / elapsed_ms as f64 * 1000.0, msg_slot_counter);
                        throughput_vec.push(ThroughputMeasurement {
                            elapsed_time: start_time.elapsed().unwrap(),
                            total_count: *recv_counter,
                        });
                        start_time = SystemTime::now();
                        *recv_counter = 0;
                    }
                    // if *recv_counter % 40000 == 0{
                        // print out port and throughput vector
                        // println!("{}", port);
                        // println!("Acceptor on port {}, throughput: {:?}", port, throughput_vec);
                    // }
                }
                default => {}
            };
        }).pull_to_push().for_each(|_| {})
    );

    let mut hf = hf.build();
    println!("Opening on port {}", port);
    hf.run_async().await.unwrap();
}
