use std::{collections::HashSet, sync::mpsc::channel};

use futures::{SinkExt, StreamExt};
use hydroflow::{
    builder::{
        prelude::{BaseSurface, PullSurface, PushSurface},
        surface::pull_handoff::HandoffPullSurface,
        HydroflowBuilder,
    },
    scheduled::{ctx::OutputPort, graph_ext::GraphExt, handoff::VecHandoff},
};
use serde::{Deserialize, Serialize};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{Receiver, Sender},
};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

#[derive(Serialize, Deserialize, Debug)]
struct EchoRequest {
    return_addr: String,
    payload: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct EchoResponse {
    payload: String,
}

fn start_echo_server() -> u16 {
    let (server_port_send, server_port_recv) = std::sync::mpsc::channel();

    // First, spin up the server.
    {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let mut builder = HydroflowBuilder::default();

                let (port, incoming_messages): (_, OutputPort<VecHandoff<EchoRequest>>) =
                    builder.hydroflow.inbound_tcp_vertex().await;
                server_port_send.send(port).unwrap();

                let handler: HandoffPullSurface<VecHandoff<EchoRequest>> =
                    builder.wrap_input(incoming_messages);

                let outbound_messages = builder.hydroflow.outbound_tcp_vertex().await;
                let outbound_messages = builder.wrap_output(outbound_messages);

                builder.add_subgraph(
                    handler
                        .flatten()
                        .map(|echo_request| {
                            (
                                echo_request.return_addr,
                                EchoResponse {
                                    payload: echo_request.payload,
                                },
                            )
                        })
                        .pivot()
                        .map(Some)
                        .reverse(outbound_messages),
                );

                builder.build().run_async().await.unwrap();
            });
        });
    }

    server_port_recv.recv().unwrap()
}

// This test sets up an echo server and runs a few connections to it.
#[test]
fn test_echo_server() {
    let (log_message, all_received_messages) = channel();

    let server_port = start_echo_server();

    // Connect to it some number of times.
    for idx in 0..3 {
        let log_message = log_message.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let mut builder = HydroflowBuilder::default();

                let (port, responses) = builder.hydroflow.inbound_tcp_vertex().await;

                let outbound_messages = builder.hydroflow.outbound_tcp_vertex().await;
                let (input, requests) = builder.hydroflow.add_input();
                builder.hydroflow.add_edge(requests, outbound_messages);

                let responses = builder.wrap_input(responses);

                builder.add_subgraph(responses.flatten().pivot().for_each(
                    move |response: EchoResponse| {
                        log_message
                            .send(format!(
                                "[CLIENT#{}] received back {:?}",
                                idx, response.payload
                            ))
                            .unwrap()
                    },
                ));

                let server_addr = format!("localhost:{}", server_port);

                input.give(Some((
                    server_addr.clone(),
                    EchoRequest {
                        return_addr: format!("localhost:{}", port),
                        payload: format!("Hello world {}!", idx),
                    },
                )));

                builder.build().run_async().await.unwrap();
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

// A simple echo client to use to exercise the server.
#[derive(Debug)]
struct EchoClient {
    port: u16,
    sender: Sender<(String, EchoRequest)>,
    receiver: Receiver<EchoResponse>,
    stop_tx: tokio::sync::watch::Sender<bool>,
}

impl EchoClient {
    async fn new() -> Self {
        let listener = TcpListener::bind("localhost:0").await.unwrap();
        Self::new_from_listener(listener).await
    }

    async fn new_from_listener(listener: TcpListener) -> Self {
        let port = listener.local_addr().unwrap().port();

        let (incoming_send, incoming_recv) = tokio::sync::mpsc::channel(1024);
        let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);

        tokio::spawn(async move {
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                let (reader, _) = socket.into_split();
                let mut reader = FramedRead::new(reader, LengthDelimitedCodec::new());
                let incoming_send = incoming_send.clone();
                let mut stop_rx = stop_rx.clone();
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            v = reader.next() => {
                                match v {
                                    Some(msg) => {
                                        // TODO(justin): figure out error handling here.
                                        let msg = msg.unwrap();
                                        let out = bincode::deserialize(&msg).unwrap();
                                        incoming_send.send(out).await.unwrap();
                                    }
                                    None => {
                                        return;
                                    }
                                }
                            }
                            _ = stop_rx.changed() => {
                                return;
                            }
                        }
                    }
                });
            }
        });

        let (outgoing_send, mut outgoing_recv) = tokio::sync::mpsc::channel(1024);
        tokio::spawn(async move {
            // Don't do anything clever with this test implementation, just re-open
            // a connection every time.
            while let Some((addr, msg)) = outgoing_recv.recv().await {
                let addr: String = addr;
                let msg: EchoRequest = msg;
                let conn = TcpStream::connect(addr).await.unwrap();
                let mut conn = FramedWrite::new(conn, LengthDelimitedCodec::new());
                let msg = bincode::serialize(&msg).unwrap();
                conn.send(msg.into()).await.unwrap();
            }
        });

        Self {
            port,
            sender: outgoing_send,
            receiver: incoming_recv,
            stop_tx,
        }
    }

    async fn send(&mut self, server_addr: String, payload: String) {
        self.sender
            .send((
                server_addr,
                EchoRequest {
                    return_addr: format!("localhost:{}", self.port),
                    payload,
                },
            ))
            .await
            .unwrap();
    }

    async fn recv(&mut self) -> Option<EchoResponse> {
        self.receiver.recv().await
    }

    fn stop(&mut self) {
        self.stop_tx.send(true).unwrap();
    }
}

impl Drop for EchoClient {
    fn drop(&mut self) {
        self.stop();
    }
}

#[test]
fn test_echo_server_reconnect() {
    let server_port = start_echo_server();
    let server_addr = format!("localhost:{}", server_port);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
        let mut client = EchoClient::new().await;

        client.send(server_addr.clone(), "abc".into()).await;
        let result = client.recv().await.unwrap();

        assert_eq!(
            result,
            EchoResponse {
                payload: "abc".into()
            }
        );

        let mut client = EchoClient::new().await;

        client.send(server_addr.clone(), "def".into()).await;
        let result = client.recv().await.unwrap();

        assert_eq!(
            result,
            EchoResponse {
                payload: "def".into()
            }
        );
    });
}
