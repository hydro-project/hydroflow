use std::net::SocketAddr;
use std::pin::Pin;

use bytes::{Bytes, BytesMut};
use clap::{Parser, ValueEnum};
use client::run_client;
use futures::stream::{SplitSink, SplitStream};
use futures::task::noop_waker;
use futures::SinkExt;
use hydroflow::lang::graph::{WriteConfig, WriteGraphType};
use hydroflow::tokio;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve, TcpFramedStream};
use multiplatform_test::multiplatform_test;
use server::run_server;
use tokio_util::codec::LengthDelimitedCodec;
use tokio_util::udp::UdpFramed;

use crate::protocol::{ServerReq, ServerResp};

mod client;
mod helpers;
mod protocol;
mod server;

#[derive(Clone, ValueEnum, Debug)]
enum Role {
    Client,
    Server,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(value_enum, long)]
    role: Role,
    #[clap(long, value_parser = ipv4_resolve)]
    addr: SocketAddr,
    #[clap(long, value_parser = ipv4_resolve)]
    server_addr: Option<SocketAddr>,
    #[clap(long)]
    graph: Option<WriteGraphType>,
    #[clap(flatten)]
    write_config: Option<WriteConfig>,
}

#[hydroflow::main]
async fn main() {
    let opts = Opts::parse();
    let addr = opts.addr;

    match opts.role {
        Role::Client => {
            let (outbound, inbound, _) = bind_udp_bytes(addr).await;
            println!("Client is bound to {:?}", addr);
            println!("Attempting to connect to server at {:?}", opts.server_addr);
            run_client(outbound, inbound, opts).await;
        }
        Role::Server => {
            run_server(opts.addr).await;
        }
    }
}

async fn send(
    outbound: &mut SplitSink<UdpFramed<LengthDelimitedCodec>, (Bytes, SocketAddr)>,
    x: ServerReq,
    addr: SocketAddr,
) {
    outbound
        .send((hydroflow::util::serialize_to_bytes(x), addr))
        .await
        .unwrap();
}

async fn read(inbound: &mut SplitStream<UdpFramed<LengthDelimitedCodec>>) -> ServerResp {
    use futures::StreamExt;

    let Some(Ok((bytes, src))) = inbound.next().await else {
        panic!()
    };

    hydroflow::util::deserialize_from_bytes(bytes).unwrap()
}

// #[multiplatform_test(hydroflow, env_tracing)]
#[hydroflow::test]
async fn test_server() {
    let server_addr_1 = "127.0.0.1:2098".parse().unwrap();
    let server_addr_2: SocketAddr = "127.0.0.1:2096".parse().unwrap();

    tokio::task::spawn_local(run_server(server_addr_1));
    // tokio::task::spawn_local(run_server(server_addr_2));
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let (mut outbound, mut inbound, _) = bind_udp_bytes("127.0.0.1:0".parse().unwrap()).await;

    send(
        &mut outbound,
        ServerReq::AddNode {
            node_id: server_addr_1,
        },
        server_addr_1,
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    send(
        &mut outbound,
        ServerReq::AddNode {
            node_id: server_addr_2,
        },
        server_addr_1,
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // send(
    //     &mut outbound,
    //     ServerReq::RemoveNode {
    //         node_id: server_addr_2,
    //     },
    //     server_addr_1,
    // )
    // .await;
    // tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    send(
        &mut outbound,
        ServerReq::ClientPut {
            key: "mykey".to_owned(),
            value: "myval".to_owned(),
        },
        server_addr_1,
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // send(
    //     &mut outbound,
    //     ServerReq::ClientPut {
    //         key: "mykey".to_owned(),
    //         value: "myval2".to_owned(),
    //     },
    //     server_addr_1,
    // )
    // .await;
    // tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    send(
        &mut outbound,
        ServerReq::ClientGet {
            key: "mykey".to_owned(),
        },
        server_addr_1,
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    println!("received from srv: {:?}", read(&mut inbound).await);

    // send(
    //     &mut outbound,
    //     ServerReq::ClientGet {
    //         key: "mykey2".to_owned(),
    //     },
    //     server_addr_1,
    // )
    // .await;
    // tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Need this sleep otherwise the last sent messages won't get processed before the whole process terminates.
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
}

#[test]
fn test() {
    use std::io::Write;

    use hydroflow::util::{run_cargo_example, wait_for_process_output};

    let (_server_1, _, mut server_1_stdout) =
        run_cargo_example("kvs_replicated_v2", "--role server --addr 127.0.0.1:2051");

    let mut server_1_output = String::new();
    wait_for_process_output(&mut server_1_output, &mut server_1_stdout, "Server live!");

    let (_client_1, mut client_1_stdin, mut client_1_stdout) = run_cargo_example(
        "kvs_replicated_v2",
        "--role client --addr 127.0.0.1:2052 --server-addr 127.0.0.1:2051",
    );

    let mut client_1_output = String::new();
    wait_for_process_output(&mut client_1_output, &mut client_1_stdout, "Client live!");

    client_1_stdin.write_all(b"PUT a,7\n").unwrap();

    // let (_server_2, _, mut server_2_stdout) = run_cargo_example(
    //     "kvs_replicated_v2",
    //     "--role server --addr 127.0.0.1:2053 --server-addr 127.0.0.1:2051",
    // );

    // let (_client_2, mut client_2_stdin, mut client_2_stdout) = run_cargo_example(
    //     "kvs_replicated_v2",
    //     "--role client --addr 127.0.0.1:2054 --server-addr 127.0.0.1:2053",
    // );

    // let mut server_2_output = String::new();
    // wait_for_process_output(&mut server_2_output, &mut server_2_stdout, "Server live!");
    // wait_for_process_output(
    //     &mut server_2_output,
    //     &mut server_2_stdout,
    //     r#"Message received PeerGossip \{ key: "a", value: "7" \} from 127\.0\.0\.1:2051"#,
    // );

    // let mut client_2_output = String::new();
    // wait_for_process_output(&mut client_2_output, &mut client_2_stdout, "Client live!");

    // client_2_stdin.write_all(b"GET a\n").unwrap();
    // wait_for_process_output(
    //     &mut client_2_output,
    //     &mut client_2_stdout,
    //     r#"Got a Response: ServerResponse \{ key: "a", value: "7" \}"#,
    // );
}
