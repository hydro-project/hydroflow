use std::time::Duration;

use tokio::time::timeout;

// cannot use hydroflow::main because connect_local_blocking causes a deadlock
#[tokio::main]
async fn main() {
    let batch_size = 8192;
    let mut flow = hydroflow_plus_test::compute_pi_local::compute_pi_runtime!(&batch_size);
    flow.meta_graph()
        .unwrap()
        .open_mermaid(&Default::default())
        .unwrap();
    timeout(Duration::from_secs(10), flow.run_async())
        .await
        .expect_err("Expected timeout");
}
