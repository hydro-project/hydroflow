use std::time::Duration;
use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink},
    serialize_to_bytes, batched_sink
};
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let to_broadcaster_port = ports
        .remove("to_broadcaster")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;

    let broadcasters = to_broadcaster_port.keys.clone();
    println!("broadcasters: {:?}", broadcasters);
    let to_broadcaster_unbatched_sink = to_broadcaster_port.into_sink();
    let (num_broadcaster_partitions, flush_every_n): (u32, usize) = 
        serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();
    let to_broadcaster_sink = batched_sink(to_broadcaster_unbatched_sink, flush_every_n, Duration::from_secs(10));


    let mut df = datalog!(
        r#"
        .input clientIn `repeat_iter(vec![()]) -> map(|_| (context.current_tick() as u32,))`
.input numBroadcasterPartitions `repeat_iter([(num_broadcaster_partitions,),])`
.async toBroadcaster `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(to_broadcaster_sink)` `null::<(u32,)>()`
        
toBroadcaster@(v%n)(v) :~ clientIn(v), numBroadcasterPartitions(n)
    "#
    );

    df.run_async().await;
}

