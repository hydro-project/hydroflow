// TODO(shadaj): rewrite type names that involve String
extern crate alloc;

// cannot use hydroflow::main because connect_local_blocking causes a deadlock
#[tokio::main]
async fn main() {
    let ports = hydroflow::util::cli::init().await;

    let joined = hydroflow_plus_test::cluster::map_reduce_runtime!(&ports);

    hydroflow::util::cli::launch_flow(joined).await;
}
