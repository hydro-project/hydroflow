use std::cell::RefCell;

use hydroflow_plus::futures::channel::mpsc;
use hydroflow_plus::futures::StreamExt;

// cannot use hydroflow::main because connect_local_blocking causes a deadlock
#[tokio::main]
async fn main() {
    let batch_size = 8192;
    let counters = RefCell::new(vec![0; 8192]);

    let (counter_sender, mut counter_receiver) = mpsc::unbounded();
    let counter_queue = RefCell::new(counter_sender);

    let _thread = tokio::spawn(async move {
        while let Some((id, count)) = counter_receiver.next().await {
            println!("node id {}: counter = {}", id, count);
        }
    });

    hydroflow_plus::launch!(
        |ports| hydroflow_plus_test::cluster::cardinality_compute_pi_runtime!(
            ports,
            &batch_size,
            &counters,
            &counter_queue,
        )
    )
    .await;
}
