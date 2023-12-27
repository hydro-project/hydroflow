// TODO(shadaj): rewrite type names that involve String
extern crate alloc;

// cannot use hydroflow::main because connect_local_blocking causes a deadlock
#[tokio::main]
async fn main() {
    hydroflow_plus::util::cli::launch(|ports| {
        hydroflow_plus_test::cluster::map_reduce_runtime!(ports)
    })
    .await;
}
