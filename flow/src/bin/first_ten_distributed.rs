#[tokio::main]
async fn main() {
    let ports = hydroflow_plus::util::cli::init().await;

    hydroflow_plus::util::cli::launch_flow(
        flow::first_ten_distributed::first_ten_distributed_runtime!(&ports),
    )
    .await;
}
