use hydro_deploy::Deployment;
use hydroflow_plus_deploy::TrybuildHost;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
    let localhost = deployment.Localhost();

    let builder = hydroflow_plus::FlowBuilder::new();
    let f = 1;
    let num_clients = 1;
    let num_clients_per_node = 1; // Change based on experiment between 1, 50, 100.
    let median_latency_window_size = 1000;
    let checkpoint_frequency = 1000; // Num log entries
    let i_am_leader_send_timeout = 5; // Sec
    let i_am_leader_check_timeout = 10; // Sec
    let i_am_leader_check_timeout_delay_multiplier = 15;

    let (proposers, acceptors, clients, replicas) = hydroflow_plus_test::cluster::paxos::paxos(
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
                .map(|_| TrybuildHost::new(localhost.clone()).rustflags(rustflags))
                .collect::<Vec<_>>(),
        )
        .with_cluster(
            &acceptors,
            (0..2 * f + 1)
                .map(|_| TrybuildHost::new(localhost.clone()).rustflags(rustflags))
                .collect::<Vec<_>>(),
        )
        .with_cluster(
            &clients,
            (0..num_clients)
                .map(|_| TrybuildHost::new(localhost.clone()).rustflags(rustflags))
                .collect::<Vec<_>>(),
        )
        .with_cluster(
            &replicas,
            (0..f + 1)
                .map(|_| TrybuildHost::new(localhost.clone()).rustflags(rustflags))
                .collect::<Vec<_>>(),
        )
        .deploy(&mut deployment);

    deployment.deploy().await.unwrap();

    deployment.start().await.unwrap();

    tokio::signal::ctrl_c().await.unwrap();
}
