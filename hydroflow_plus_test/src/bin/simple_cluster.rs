// cannot use hydroflow::main because connect_local_blocking causes a deadlock
#[tokio::main]
async fn main() {
    let subgraph_id: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let ports = hydroflow::util::cli::init().await;

    let joined = hydroflow_plus_test::cluster::simple_cluster_runtime!(&ports, subgraph_id);

    hydroflow::util::cli::launch_flow(joined).await;
}
