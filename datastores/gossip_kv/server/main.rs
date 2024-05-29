use std::time::Duration;

use hydroflow::{hydroflow_syntax, tokio};

#[allow(dead_code)]
mod model;

#[hydroflow::main]
async fn main() {
    let mut server = hydroflow_syntax! {

        source_interval(Duration::from_secs(5)) -> for_each(|_| println!("Coming soon!"));

    };

    server.run_async().await;
}
