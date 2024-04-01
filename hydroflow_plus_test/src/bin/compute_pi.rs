use std::cell::RefCell;

use hydroflow_plus::futures::StreamExt;

// cannot use hydroflow::main because connect_local_blocking causes a deadlock
#[tokio::main]
async fn main() {
    let batch_size = 8192;

    hydroflow_plus::launch!(|ports| hydroflow_plus_test::cluster::compute_pi_runtime!(
        ports,
        &batch_size,
    ))
    .await;
}
