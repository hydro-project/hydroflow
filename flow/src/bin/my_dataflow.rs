// cannot use hydroflow::main because connect_local_blocking causes a deadlock
#[tokio::main]
async fn main() {
    let node_id: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let ports = hydroflow_plus::util::cli::init().await;

    let joined = flow::my_dataflow_runtime!(&ports, node_id);

    hydroflow_plus::util::cli::launch_flow(joined).await;
}
