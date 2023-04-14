use futures::SinkExt;
use futures::StreamExt;
use hydroflow::serde::Deserialize;
use hydroflow::serde::Serialize;
use hydroflow::tokio;
use hydroflow::util::cli::{ConnectedBidi, ConnectedSink, ConnectedSource};
use hydroflow::util::deserialize_from_bytes;
use hydroflow::util::serialize_to_bytes;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc;
use std::thread;
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

    let atomic_counter = Arc::new(AtomicU64::new(0));
    let atomic_borrow = atomic_counter.clone();
    let (latency_sender, latency_receiver) = mpsc::channel::<u128>();
    thread::spawn(move || {
        let mut last_instant = Instant::now();
        loop {
            thread::sleep(std::time::Duration::from_millis(100));
            let now = Instant::now();
            let counter = atomic_borrow.swap(0, std::sync::atomic::Ordering::Relaxed);
            let elapsed = now - last_instant;
            last_instant = now;
            println!("throughput,{},{}", counter, elapsed.as_secs_f64());

            while let Ok(latency) = latency_receiver.try_recv() {
                println!("latency,{}", latency);
            }
        }
    });

    let mut local_belief = HashMap::new();
    loop {
        let id = rand::random::<u64>() % 1024;
        let increment = rand::random::<bool>();
        let orig_count = local_belief.entry(id).or_insert(0);
        let start = Instant::now();
        start_node
            .send(
                serialize_to_bytes(IncrementRequest {
                    tweet_id: id,
                    likes: if increment { 1 } else { -1 },
                }),
            )
            .await
            .unwrap();

        let updated = deserialize_from_bytes::<(u64, i32)>(end_node.next().await.unwrap().unwrap()).unwrap();

        latency_sender.send(start.elapsed().as_micros()).unwrap();

        *orig_count = updated.1;

        atomic_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}
