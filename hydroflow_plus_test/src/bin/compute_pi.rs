use std::cell::RefCell;
use hydroflow_plus::util::cli::init_no_ack_start;
use hydroflow_plus::util::cli::launch_flow;

// cannot use hydroflow::main because connect_local_blocking causes a deadlock
#[tokio::main]
async fn main() {
    // TODO: Figure out the number of counters we need
    let counters = RefCell::new(vec![0; 8192]);
    let ports = init_no_ack_start().await;
    let flow = hydroflow_plus_test::cluster::compute_pi_runtime!(&ports, &counters);

    println!("ack start");

    launch_flow(flow).await;
}
