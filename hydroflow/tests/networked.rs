use std::{collections::HashSet, sync::mpsc::channel};

use hydroflow::scheduled::{
    ctx::RecvCtx, graph::Hydroflow, graph_ext::GraphExt, handoff::VecHandoff,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct EchoRequest {
    return_addr: String,
    payload: String,
}

#[derive(Serialize, Deserialize)]
struct EchoResponse {
    payload: String,
}

// This test sets up an echo server and runs a few connections to it.
#[test]
fn test_echo_server() {
    let (log_message, all_received_messages) = channel();

    let (server_port_send, server_port_recv) = std::sync::mpsc::channel();

    // First, spin up the server.
    {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let mut df = Hydroflow::new();

                let (port, incoming_messages) = df.inbound_tcp_vertex().await;
                let outbound_messages = df.outbound_tcp_vertex().await;

                server_port_send.send(port).unwrap();

                let (receiver_port, sender_port) =
                    df.add_inout(move |_ctx, recv: &RecvCtx<VecHandoff<EchoRequest>>, send| {
                        for msg in recv.take_inner() {
                            send.give(Some((
                                msg.return_addr,
                                EchoResponse {
                                    payload: msg.payload,
                                },
                            )));
                        }
                    });
                df.add_edge(incoming_messages, receiver_port);

                df.add_edge(sender_port, outbound_messages);

                df.run_async().await.unwrap();
            });
        });
    }

    let server_port = server_port_recv.recv().unwrap();

    // Connect to it some number of times.
    for idx in 0..3 {
        let log_message = log_message.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let mut df = Hydroflow::new();

                let (port, incoming_messages) = df.inbound_tcp_vertex().await;
                let outbound_messages = df.outbound_tcp_vertex().await;

                let (input, given_inputs) = df.add_input();
                df.add_edge(given_inputs, outbound_messages);

                let receiver_port =
                    df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<EchoResponse>>| {
                        for v in recv.take_inner() {
                            log_message
                                .send(format!("[CLIENT#{}] received back {:?}", idx, v.payload))
                                .unwrap();
                        }
                    });

                df.add_edge(incoming_messages, receiver_port);

                let server_addr = format!("localhost:{}", server_port);

                input.give(Some((
                    server_addr.clone(),
                    EchoRequest {
                        return_addr: format!("localhost:{}", port),
                        payload: format!("Hello world {}!", idx),
                    },
                )));

                df.run_async().await.unwrap();
            });
        });
    }

    let expected_messages: HashSet<String> = [
        "[CLIENT#0] received back \"Hello world 0!\"".into(),
        "[CLIENT#1] received back \"Hello world 1!\"".into(),
        "[CLIENT#2] received back \"Hello world 2!\"".into(),
    ]
    .into_iter()
    .collect();

    let mut actual_messages = HashSet::new();
    while actual_messages.len() < expected_messages.len() {
        actual_messages.insert(all_received_messages.recv().unwrap());
    }

    assert_eq!(expected_messages, actual_messages);
}
