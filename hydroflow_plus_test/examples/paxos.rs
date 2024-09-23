use std::sync::Arc;

use hydro_deploy::gcp::GcpNetwork;
use hydro_deploy::{Deployment, Host};
use hydroflow_plus::deploy::TrybuildHost;
use tokio::sync::RwLock;

type HostCreator = Box<dyn Fn(&mut Deployment) -> Arc<dyn Host>>;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
    let host_arg = std::env::args().nth(1).unwrap_or_default();

    let create_host: HostCreator = if host_arg == *"gcp" {
        let project = std::env::args().nth(2).unwrap();
        let network = Arc::new(RwLock::new(GcpNetwork::new(&project, None)));

        Box::new(move |deployment| -> Arc<dyn Host> {
            deployment
                .GcpComputeEngineHost()
                .project(&project)
                .machine_type("n2-highcpu-2")
                .image("debian-cloud/debian-11")
                .region("us-west1-a")
                .network(network.clone())
                .add()
        })
    } else {
        let localhost = deployment.Localhost();
        Box::new(move |_| -> Arc<dyn Host> { localhost.clone() })
    };

    let builder = hydroflow_plus::FlowBuilder::new();
    let f = 1;
    let num_clients = 1;
    let num_clients_per_node = 100; // Change based on experiment between 1, 50, 100.
    let median_latency_window_size = 1000;
    let checkpoint_frequency = 1000; // Num log entries
    let i_am_leader_send_timeout = 5; // Sec
    let i_am_leader_check_timeout = 10; // Sec
    let i_am_leader_check_timeout_delay_multiplier = 15;

    let (proposers, acceptors, clients, replicas) =
        hydroflow_plus_test::cluster::paxos_bench::paxos_bench(
            &builder,
            f,
            num_clients_per_node,
            median_latency_window_size,
            checkpoint_frequency,
            i_am_leader_send_timeout,
            i_am_leader_check_timeout,
            i_am_leader_check_timeout_delay_multiplier,
        );

    let rustflags = "-C opt-level=3 -C codegen-units=1 -C strip=none -C debuginfo=2 -C lto=off";

    let _nodes = builder
        .with_default_optimize()
        .with_cluster(
            &proposers,
            (0..f + 1)
                .map(|_| TrybuildHost::new(create_host(&mut deployment)).rustflags(rustflags))
                .collect::<Vec<_>>(),
        )
        .with_cluster(
            &acceptors,
            (0..2 * f + 1)
                .map(|_| TrybuildHost::new(create_host(&mut deployment)).rustflags(rustflags))
                .collect::<Vec<_>>(),
        )
        .with_cluster(
            &clients,
            (0..num_clients)
                .map(|_| TrybuildHost::new(create_host(&mut deployment)).rustflags(rustflags))
                .collect::<Vec<_>>(),
        )
        .with_cluster(
            &replicas,
            (0..f + 1)
                .map(|_| TrybuildHost::new(create_host(&mut deployment)).rustflags(rustflags))
                .collect::<Vec<_>>(),
        )
        .deploy(&mut deployment);

    deployment.deploy().await.unwrap();

    deployment.start().await.unwrap();

    tokio::signal::ctrl_c().await.unwrap();
}
