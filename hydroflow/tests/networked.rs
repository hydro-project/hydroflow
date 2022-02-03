use std::{
    collections::{HashMap, HashSet},
    sync::{mpsc::channel, Arc, Mutex},
    time::Duration,
};

use futures::{SinkExt, StreamExt};
use hydroflow::{
    builder::{
        prelude::{BaseSurface, PullSurface, PushSurface},
        surface::{
            exchange::Exchange, pull_handoff::HandoffPullSurface, pull_iter::IterPullSurface,
            push_handoff::HandoffPushSurfaceReversed,
        },
        HydroflowBuilder,
    },
    lang::collections::Iter,
    scheduled::{graph_ext::GraphExt, handoff::VecHandoff, port::OutputPort},
};
use rand::Rng;
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
                let (input, requests) = builder
                    .hydroflow
                    .add_input::<_, VecHandoff<(String, EchoRequest)>>();
                // TODO(mingwei): fix this once network methods take ports as arguments instead of returning them.
                // // builder.hydroflow.add_edge(requests, outbound_messages);
                builder.hydroflow.add_subgraph_in_out(
                    requests,
                    outbound_messages,
                    |_ctx, recv, send| {
                        send.give(Iter(recv.take_inner().into_iter()));
                    },
                );

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

async fn open_connection(
    builder: &mut HydroflowBuilder,
) -> (
    u16,
    HandoffPullSurface<VecHandoff<(u64, String)>>,
    HandoffPushSurfaceReversed<
        VecHandoff<(String, (u64, String))>,
        Option<(String, (u64, String))>,
    >,
) {
    let (port, messages) = builder
        .hydroflow
        .inbound_tcp_vertex::<(u64, String)>()
        .await;
    let inbound_messages = builder.wrap_input(messages);
    let outbound_messages = builder
        .hydroflow
        .outbound_tcp_vertex::<(u64, String)>()
        .await;
    let outbound_messages = builder.wrap_output(outbound_messages);
    (port, inbound_messages, outbound_messages)
}

#[test]
fn test_exchange() {
    // This test sets up a dataflow with a number of participants each having a
    // random subset of data, then does an exchange, and has each participant
    // run a local join on the exchanged data.

    #[derive(Debug, Clone)]
    struct Participant {
        english: Vec<(u64, String)>,
        french: Vec<(u64, String)>,
        id: u64,
    }

    const NUM_PARTICIPANTS: u64 = 3;

    let mut local_data: Vec<_> = (0..NUM_PARTICIPANTS)
        .map(|_| (Vec::new(), Vec::new()))
        .collect();

    let data = [
        (0, "zero", "zero"),
        (1, "one", "une"),
        (2, "two", "deux"),
        (3, "three", "trois"),
        (4, "four", "quatre"),
        (5, "five", "cinq"),
        (6, "six", "six"),
        (7, "seven", "sept"),
        (8, "eight", "huit"),
        (9, "nine", "neuf"),
        (10, "ten", "dix"),
    ];

    // Shuffle the data between the participants.
    let mut rng = rand::thread_rng();
    for (val, english, french) in &data {
        // Pick someone to take the English word.
        let english_recepient = rng.gen_range(0..NUM_PARTICIPANTS) as usize;
        local_data[english_recepient]
            .0
            .push((*val, (*english).to_owned()));

        let french_recepient = rng.gen_range(0..NUM_PARTICIPANTS) as usize;
        local_data[french_recepient]
            .1
            .push((*val, (*french).to_owned()));
    }

    let participants: Vec<_> = local_data
        .into_iter()
        .enumerate()
        .map(|(i, (english, french))| Participant {
            english,
            french,
            id: i as u64,
        })
        .collect();

    let english_address_book = Arc::new(Mutex::new(HashMap::new()));
    let french_address_book = Arc::new(Mutex::new(HashMap::new()));

    let (receipts_tx, receipts_rx) = std::sync::mpsc::channel();

    for p in participants {
        let english_address_book = english_address_book.clone();
        let french_address_book = french_address_book.clone();
        let receipts_tx = receipts_tx.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let mut builder = HydroflowBuilder::default();

                // Set up English communication.
                let (port, english_inbound_messages, english_outbound_messages) =
                    open_connection(&mut builder).await;

                english_address_book
                    .lock()
                    .unwrap()
                    .insert(p.id, format!("localhost:{}", port));

                // Set up French communication.
                let (port, french_inbound_messages, french_outbound_messages) =
                    open_connection(&mut builder).await;

                french_address_book
                    .lock()
                    .unwrap()
                    .insert(p.id, format!("localhost:{}", port));

                // Wait for everyone to fill in their connection info.
                while english_address_book.lock().unwrap().len() < NUM_PARTICIPANTS as usize
                    || french_address_book.lock().unwrap().len() < NUM_PARTICIPANTS as usize
                {
                    std::thread::sleep(Duration::from_millis(30));
                }

                // Build the French exchanged input.
                let french_address_book = french_address_book.lock().unwrap().clone();
                let french = IterPullSurface::new(p.french.into_iter()).exchange(
                    &mut builder,
                    french_address_book,
                    french_inbound_messages.flatten(),
                    p.id,
                    french_outbound_messages,
                );

                // Build the English exchanged input.
                let english_address_book = english_address_book.lock().unwrap().clone();
                let english = IterPullSurface::new(p.english.into_iter()).exchange(
                    &mut builder,
                    english_address_book,
                    english_inbound_messages.flatten(),
                    p.id,
                    english_outbound_messages,
                );

                let join = english.join(french);

                // Join them.
                builder.add_subgraph(join.pivot().for_each(move |(v, english, french)| {
                    receipts_tx.send((v, english, french)).unwrap();
                }));

                builder.build().run_async().await.unwrap();
            });
        });
    }

    let mut out = Vec::new();
    while out.len() < data.len() {
        out.push(receipts_rx.recv().unwrap());
    }
    out.sort_unstable();

    // If all went well, we should match up the English words with the French words.
    assert_eq!(
        &out,
        &[
            (0, "zero".to_owned(), "zero".to_owned()),
            (1, "one".to_owned(), "une".to_owned()),
            (2, "two".to_owned(), "deux".to_owned()),
            (3, "three".to_owned(), "trois".to_owned()),
            (4, "four".to_owned(), "quatre".to_owned()),
            (5, "five".to_owned(), "cinq".to_owned()),
            (6, "six".to_owned(), "six".to_owned()),
            (7, "seven".to_owned(), "sept".to_owned()),
            (8, "eight".to_owned(), "huit".to_owned()),
            (9, "nine".to_owned(), "neuf".to_owned()),
            (10, "ten".to_owned(), "dix".to_owned())
        ]
    );
}
