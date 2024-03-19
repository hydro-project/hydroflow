use std::net::SocketAddr;

use clap::{Parser, ValueEnum};
use client::run_client;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve};
use hydroflow_lang::graph::{WriteConfig, WriteGraphType};
use server::run_server;
use crate::randomized_gossiping_server::run_gossiping_server;

mod client;
mod protocol;
mod server;
mod randomized_gossiping_server;

#[derive(Clone, ValueEnum, Debug)]
enum Role {
    Client,
    Server,
    GossipingServer1,
    GossipingServer2,
    GossipingServer3,
    GossipingServer4,
    GossipingServer5,
}

pub fn default_server_address() -> SocketAddr {
    ipv4_resolve("localhost:54321").unwrap()
}


#[derive(Parser, Debug)]
struct Opts {
    #[clap(long)]
    name: String,
    #[clap(value_enum, long)]
    role: Role,
    #[clap(long, value_parser = ipv4_resolve)]
    address: Option<SocketAddr>,
    #[clap(long)]
    graph: Option<WriteGraphType>,
    #[clap(flatten)]
    write_config: Option<WriteConfig>,
}

#[hydroflow::main]
async fn main() {
    let opts = Opts::parse();

    match opts.role {
        Role::Client => {
            run_client(opts).await;
        }
        Role::Server => {
            run_server(opts).await;
        }
        Role::GossipingServer1 | Role::GossipingServer2 | Role::GossipingServer3 | Role::GossipingServer4 | Role::GossipingServer5 => {
            run_gossiping_server(opts).await
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

    // wait 100ms so we don't drop a packet
    let hundo_millis = std::time::Duration::from_millis(100);
    std::thread::sleep(hundo_millis);

    client1_input.write_all(b"Hello\n").unwrap();

    wait_for_process_output(
        &mut client2_output_so_far,
        &mut client2_output,
        ".*, .* client1: Hello",
    );
}
