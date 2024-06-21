#[tokio::main]
async fn main() {
    flow::first_ten::first_ten_runtime!().run_async().await;
}
