use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
};
use hydroflow::tokio_stream::wrappers::IntervalStream;
use hydroflow_datalog::datalog;
use tokio::time::{interval_at, Duration, Instant};

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let to_replica_port = ports
        .remove("to_replica")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;

    let peers = to_replica_port.keys.clone();
    println!("peers: {:?}", peers);
    let to_replica_sink = to_replica_port.into_sink();

    let from_replica_source = ports
        .remove("from_replica")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let frequency = 1;
    let start = Instant::now() + Duration::from_secs(frequency);
    let periodic_source = IntervalStream::new(interval_at(start, Duration::from_secs(frequency)));

    let mut df = datalog!(
        r#"
        .input clientIn `repeat_iter(vec![()]) -> map(|_| (context.current_tick(),))`
.output stdout `for_each(|s:(u32,)| println!("committed: {:?}", s))`
.input replicas `repeat_iter(peers.clone()) -> map(|p| (p,))`

.input periodic `source_stream(periodic_source) -> map(|_| ())`
.output throughputOut `for_each(|(num,):(u32,)| println!("committed {:?} entries", num))`

.async voteToReplica `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(to_replica_sink)` `null::<(u32,)>()`
.async voteFromReplica `null::<(u32,)>()` `source_stream(from_replica_source) -> map(|v| deserialize_from_bytes::<(u32,)>(v.unwrap()).unwrap())`

voteToReplica@addr(v) :~ clientIn(v), replicas(addr)
// stdout(v) :- voteFromReplica(l, v)

NumCommits(count(v)) :- voteFromReplica(v)
totalCommitted(new) :+ !totalCommitted(prev), NumCommits(new)
totalCommitted(prev) :+ totalCommitted(prev), !NumCommits(new)
totalCommitted(prev + new) :+ totalCommitted(prev), NumCommits(new)
throughputOut(num) :- totalCommitted(num), periodic()
    "#
    );

    df.run_async().await;
}
