use std::{collections::HashSet, sync::mpsc::channel};

use hydroflow::{
    lang::collections::Iter,
    scheduled::{
        ctx::RecvCtx, graph::Hydroflow, graph_demux::GraphDemux, graph_ext::GraphExt,
        handoff::VecHandoff, net::Message,
    },
};
use tokio::net::{TcpListener, TcpStream};

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

                let (receiver_port, sender_port) = df.add_inout(move |_ctx, recv, send| {
                    for msg in recv.take_inner() {
                        let mut msg: Message = msg;
                        let addr = format!("localhost:{}", msg.address);
                        msg.address = port.try_into().unwrap();
                        send.give(Some((addr, msg)));
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
                    df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<Message>>| {
                        for v in recv.take_inner() {
                            log_message
                                .send(format!(
                                    "[CLIENT#{}] received back {:?}",
                                    idx,
                                    String::from_utf8(v.batch.to_vec()).unwrap()
                                ))
                                .unwrap();
                        }
                    });

                df.add_edge(incoming_messages, receiver_port);

                let server_addr = format!("localhost:{}", server_port);

                input.give(Some((
                    server_addr.clone(),
                    Message {
                        address: port.into(),
                        batch: format!("Hello world {}!", idx).into(),
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

#[test]
fn test_networked() {
    let (port_sender, port_receiver) = channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let mut df = Hydroflow::new();

            let stream = TcpListener::bind("localhost:0").await.unwrap();
            let addr = stream.local_addr().unwrap();

            port_sender.send(addr.port()).unwrap();

            let (stream, _) = stream.accept().await.unwrap();
            let network_send = df.add_write_tcp_stream(stream);

            let (input, out) = df.add_input();

            input.give(Iter(
                vec![Message {
                    address: 0,
                    batch: vec![1, 2, 3, 4].into(),
                }]
                .into_iter(),
            ));

            df.add_edge(out, network_send);

            df.run_async().await.unwrap();
        });
    });

    let (send, recv) = channel();

    std::thread::spawn(move || {
        let port = port_receiver.recv().unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let mut df = Hydroflow::new();

            let stream = TcpStream::connect(format!("localhost:{}", port))
                .await
                .unwrap();
            let network_recv = df.add_read_tcp_stream(stream);

            let input = df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<_>>| {
                for v in recv.take_inner() {
                    send.send(v).unwrap();
                }
            });

            let (demux, input_port) = df.add_demux::<_, _, _, VecHandoff<_>>(|_| ());

            df.add_edge(network_recv, input_port);
            df.add_demux_edge(&demux, (), input);

            df.run_async().await.unwrap();
        });
    });

    let val = recv.recv().unwrap();
    assert_eq!(
        val,
        Message {
            address: 0,
            batch: vec![1, 2, 3, 4].into(),
        }
    );
}
