#[tokio::main]
async fn main() {
    hydroflow_plus::util::cli::launch(|ports| {
        flow::first_ten_distributed::first_ten_distributed_runtime!(&ports)
    })
    .await;
}
