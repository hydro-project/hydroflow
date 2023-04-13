use futures::SinkExt;
use futures::StreamExt;
use hydroflow::bytes::Bytes;
use hydroflow::hydroflow_syntax;
use hydroflow::serde::Deserialize;
use hydroflow::serde::Serialize;
use hydroflow::tokio;
use hydroflow::util::cli::{ConnectedBidi, ConnectedSink, ConnectedSource};
use hydroflow::util::{deserialize_from_bytes, serialize_to_bytes};
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct IncrementRequest {
    tweet_id: u64,
    likes: i32,
}

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let mut start_node = ports
        .remove("increment_start_node")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_sink();

    let mut end_node = ports
        .remove("end_node_query")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    loop {
        let start = Instant::now();
        start_node.send(
            serde_json::to_string(&IncrementRequest {
                tweet_id: 0,
                likes: 1,
            }).unwrap().into_bytes().into()
        ).await.unwrap();

        end_node.next().await;
        println!("latency,{:?}", start.elapsed().as_micros());
    }
}
