// cannot use hydroflow::main because connect_local_blocking causes a deadlock
#[tokio::main]
async fn main() {
    let ports = hydroflow::util::cli::init().await;

    hydroflow::util::cli::launch_flow(
        hydroflow_plus_test::first_ten::first_ten_distributed_runtime!(&ports),
    )
    .await;
}
