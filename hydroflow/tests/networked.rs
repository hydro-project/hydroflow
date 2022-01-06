use std::{
    collections::{HashMap, HashSet},
    sync::{mpsc::channel, Arc, Mutex},
    time::Duration,
};

use futures::{SinkExt, StreamExt};
use hydroflow::{
    lang::collections::Iter,
    scheduled::{
        ctx::RecvCtx, graph::Hydroflow, graph_demux::GraphDemux, graph_ext::GraphExt,
        handoff::VecHandoff, net::Message,
    },
};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

#[test]
fn test_star_vertex() {
    // This test constructs 3 Hydroflow graphs that all send messages to each other.
    const NUM_PARTICIPANTS: u32 = 3;

    // Map participant IDs onto ports.
    let addresses = Arc::new(Mutex::new(HashMap::<u32, u16>::new()));

    // Channel that we're going to log everything that happens in, so we can
    // assert that the set of messages is sane at the end.
    let (log_message, all_received_messages) = channel();

    let mut handles = Vec::new();

    // Each iteration of this loops spins up another Hydroflow graph.
    for id in 0..NUM_PARTICIPANTS {
        let addresses = addresses.clone();
        let log_message = log_message.clone();
        handles.push(std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                // First, start listening on a port.
                let listener = TcpListener::bind("localhost:0").await.unwrap();
                let addr = listener.local_addr().unwrap();
                let port = addr.port();

                // Put _our_ address into the map. This is in lieu of a name server.
                addresses.lock().unwrap().insert(id, port);

                // Wait for all the addresses to be populated.
                while addresses.lock().unwrap().len() < NUM_PARTICIPANTS.try_into().unwrap() {
                    std::thread::sleep(Duration::from_millis(30));
                }

                let addresses = addresses.lock().unwrap().clone();

                let mut df = Hydroflow::new();

                // Set up the messages we will send to other participants.
                let (mut conns, egress_edge) = df.add_egress_vertex();
                let (incoming_send, incoming_messages) = futures::channel::mpsc::channel(1024);

                // To keep things simple for now, every graph is going to
                // connect to every other graph and have a fresh connection for
                // each unidirectional bit of communication.

                // TODO(justin): reuse the TCP connection to go the other direction too.

                // Listen to incoming connections and spawn a task for each one
                // that will shove their messages into a channel.
                // TODO(justin): good candidate for extraction.
                tokio::spawn(async move {
                    loop {
                        let (socket, _) = listener.accept().await.unwrap();
                        let (reader, _) = socket.into_split();
                        let mut reader = FramedRead::new(reader, LengthDelimitedCodec::new());
                        let mut incoming_send = incoming_send.clone();
                        tokio::spawn(async move {
                            loop {
                                while let Some(msg) = reader.next().await {
                                    // TODO(justin): figure out error handling here.
                                    let msg = msg.unwrap();
                                    let msg = Message::decode(&msg.freeze());
                                    incoming_send.feed(msg).await.unwrap();
                                }
                            }
                        });
                    }
                });

                // Connect to each other server.
                tokio::spawn(async move {
                    for (other_id, other_port) in addresses.into_iter() {
                        if other_id == id {
                            continue;
                        }

                        let stream = TcpStream::connect(format!("localhost:{}", other_port))
                            .await
                            .unwrap();

                        let (_, writer) = stream.into_split();

                        conns.feed((other_id, writer)).await.unwrap();
                    }
                    conns.flush().await.unwrap();
                });

                // When we receive a message, log it so it can be asserted at the end of the test.
                let receiver_port =
                    df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<Message>>| {
                        for v in recv.take_inner() {
                            log_message
                                .send(format!(
                                    "[{}] received {:?}",
                                    id,
                                    String::from_utf8(v.batch.to_vec()).unwrap()
                                ))
                                .unwrap();
                        }
                    });

                let incoming_messages = df.add_input_from_stream(incoming_messages.map(Some));
                df.add_edge(incoming_messages, receiver_port);

                let (input, send_port) = df.add_input();

                df.add_edge(send_port, egress_edge);

                // Send a message to each other graph.
                for i in 0..NUM_PARTICIPANTS {
                    if i == id {
                        continue;
                    }
                    input.give(Some((
                        i,
                        Message {
                            address: 0,
                            batch: format!("Hello from graph {}", id).into(),
                        },
                    )))
                }

                df.run_async().await.unwrap();
            })
        }));
    }

    let expected_messages: HashSet<String> = [
        "[0] received \"Hello from graph 1\"".into(),
        "[0] received \"Hello from graph 2\"".into(),
        "[1] received \"Hello from graph 0\"".into(),
        "[1] received \"Hello from graph 2\"".into(),
        "[2] received \"Hello from graph 0\"".into(),
        "[2] received \"Hello from graph 1\"".into(),
    ]
    .into_iter()
    .collect();

    let mut actual_messages = HashSet::new();
    while actual_messages.len() < expected_messages.len() {
        actual_messages.insert(all_received_messages.recv().unwrap());
    }

    assert_eq!(actual_messages, expected_messages);
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
