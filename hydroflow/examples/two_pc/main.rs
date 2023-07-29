use std::net::SocketAddr;
use std::path::Path;

use clap::{Parser, ValueEnum};
use coordinator::run_coordinator;
use hydroflow::tokio;
use hydroflow::util::{bind_udp_bytes, ipv4_resolve};
use serde::Deserialize;
use subordinate::run_subordinate;

mod coordinator;
mod helpers;
mod protocol;
mod subordinate;

/// This is a remedial 2PC implementation.

#[derive(Clone, ValueEnum, Debug)]
enum Role {
    Coordinator,
    Subordinate,
}

#[derive(Clone, ValueEnum, Debug)]
enum GraphType {
    Mermaid,
    Dot,
    Json,
}

#[derive(Parser, Debug)]
struct Opts {
    #[clap(long)]
    path: String,
    #[clap(value_enum, long)]
    role: Role,
    #[clap(long, value_parser = ipv4_resolve)]
    addr: SocketAddr,
    #[clap(value_enum, long)]
    graph: Option<GraphType>,
}

#[derive(Deserialize, Debug)]
struct Addresses {
    coordinator: String,
    subordinates: Vec<String>,
}

#[hydroflow::main]
async fn main() {
    let opts = Opts::parse();
    let path = Path::new(&opts.path);
    let addr = opts.addr;

    match opts.role {
        Role::Coordinator => {
            let (outbound, inbound, _) = bind_udp_bytes(addr).await;
            run_coordinator(outbound, inbound, path, opts.graph.clone()).await;
        }
        Role::Subordinate => {
            let (outbound, inbound, _) = bind_udp_bytes(addr).await;
            run_subordinate(outbound, inbound, path, opts.graph.clone()).await;
        }
    }
}

#[test]
fn test() {
    use std::io::Write;

    use hydroflow::util::{run_cargo_example, wait_for_process_output};

    let members_path = format!(
        "{}/examples/two_pc/members.json",
        env!("CARGO_MANIFEST_DIR")
    );

    let (_coordinator, mut coordinator_stdin, mut coordinator_stdout) = run_cargo_example(
        "two_pc",
        &format!("--path {members_path} --role coordinator --addr 127.0.0.1:12346"),
    );

    let (_subordinate1, _, mut subordinate1_stdout) = run_cargo_example(
        "two_pc",
        &format!("--path {members_path} --role subordinate --addr 127.0.0.1:12347"),
    );

    let (_subordinate2, _, mut subordinate2_stdout) = run_cargo_example(
        "two_pc",
        &format!("--path {members_path} --role subordinate --addr 127.0.0.1:12348"),
    );

    let (_subordinate3, _, mut subordinate3_stdout) = run_cargo_example(
        "two_pc",
        &format!("--path {members_path} --role subordinate --addr 127.0.0.1:12349"),
    );

    let mut coordinator_output = String::new();
    wait_for_process_output(
        &mut coordinator_output,
        &mut coordinator_stdout,
        "Coordinator live!",
    );

    let mut subordinate1_output = String::new();
    wait_for_process_output(
        &mut subordinate1_output,
        &mut subordinate1_stdout,
        "Subordinate live!",
    );

    let mut subordinate2_output = String::new();
    wait_for_process_output(
        &mut subordinate2_output,
        &mut subordinate2_stdout,
        "Subordinate live!",
    );

    let mut subordinate3_output = String::new();
    wait_for_process_output(
        &mut subordinate3_output,
        &mut subordinate3_stdout,
        "Subordinate live!",
    );

    coordinator_stdin.write_all(b"1\n").unwrap();

    let mut coordinator_output = String::new();
    wait_for_process_output(
        &mut coordinator_output,
        &mut coordinator_stdout,
        r#"Sending CoordMsg \{ xid: 1, mtype: Prepare \} to 127.0.0.1:12347"#,
    );
    wait_for_process_output(
        &mut coordinator_output,
        &mut coordinator_stdout,
        r#"Sending CoordMsg \{ xid: 1, mtype: Prepare \} to 127.0.0.1:12348"#,
    );
    wait_for_process_output(
        &mut coordinator_output,
        &mut coordinator_stdout,
        r#"Sending CoordMsg \{ xid: 1, mtype: Prepare \} to 127.0.0.1:12349"#,
    );

    // One of two things can happen now, all 3 members commit or at least one of them aborts the transaction.
    // In the case of all 3 commits, then 3 "Commit" messages will be printed, in the case of an aborted transaction then 'Ended' will get printed, so:
    wait_for_process_output(
        &mut coordinator_output,
        &mut coordinator_stdout,
        r#"(Received SubordResponse \{ xid: 1, mtype: Commit \}|Received SubordResponse \{ xid: 1, mtype: Ended \})"#,
    );
}
