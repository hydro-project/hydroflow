// cannot use hydroflow::main because connect_local_blocking causes a deadlock
#[tokio::main]
async fn main() {
    hydroflow_plus::util::cli::launch(|ports| {
        hydroflow_plus_test::cluster::many_to_many_runtime!(ports)
    })
    .await;
}
