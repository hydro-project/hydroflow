use std::net::SocketAddr;

use clap::{Parser, ValueEnum};
use client::run_client;
use hydroflow::tokio;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve};
use hydroflow_lang::graph::{WriteConfig, WriteGraphType};
use server::run_server;

mod client;
mod protocol;
mod server;

#[derive(Clone, ValueEnum, Debug)]
enum Role {
    Client,
    Server,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(long)]
    name: String,
    #[clap(value_enum, long)]
    role: Role,
    #[clap(long, value_parser = ipv4_resolve)]
    addr: Option<SocketAddr>,
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
    // if no addr was provided, we ask the OS to assign a local port by passing in "localhost:0"
    let addr = opts
        .addr
        .unwrap_or_else(|| ipv4_resolve("localhost:0").unwrap());

    // allocate `outbound` sink and `inbound` stream
    let (outbound, inbound, addr) = bind_udp_bytes(addr).await;
    println!("Listening on {:?}", addr);

    match opts.role {
        Role::Client => {
            run_client(outbound, inbound, opts).await;
        }
        Role::Server => {
            run_server(outbound, inbound, opts).await;
        }
    }
}

#[test]
fn test() {
    use std::io::Write;

    use hydroflow::util::{run_cargo_example, wait_for_process_output};

    let (_server, _, mut server_output) =
        run_cargo_example("chat", "--role server --name server --addr 127.0.0.1:11247");

    let mut server_output_so_far = String::new();
    wait_for_process_output(
        &mut server_output_so_far,
        &mut server_output,
        "Server live!",
    );

    let (_client1, mut client1_input, mut client1_output) = run_cargo_example(
        "chat",
        "--role client --name client1 --server-addr 127.0.0.1:11247",
    );

    let (_client2, _, mut client2_output) = run_cargo_example(
        "chat",
        "--role client --name client2 --server-addr 127.0.0.1:11247",
    );

    let mut client1_output_so_far = String::new();
    let mut client2_output_so_far = String::new();

    wait_for_process_output(
        &mut client1_output_so_far,
        &mut client1_output,
        "Client live!",
    );
    wait_for_process_output(
        &mut client2_output_so_far,
        &mut client2_output,
        "Client live!",
    );

    client1_input.write_all(b"Hello\n").unwrap();

    wait_for_process_output(
        &mut client2_output_so_far,
        &mut client2_output,
        ".*, .* client1: Hello",
    );
}
