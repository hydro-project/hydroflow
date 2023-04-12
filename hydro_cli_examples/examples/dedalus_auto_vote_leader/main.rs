use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink},
    serialize_to_bytes,
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
    let to_broadcaster_sink = to_broadcaster_port.into_sink();
    let (num_broadcaster_partitions, flush_every_n): (u32,u32) = 
        serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();


    let mut df = datalog!(
        r#"
        .input clientIn `repeat_iter(vec![()]) -> map(|_| (context.current_tick() as u32,))`
.input numBroadcasterPartitions `repeat_iter([(num_broadcaster_partitions,),])`
.input flushEveryN `repeat_iter([(flush_every_n,),])`
.async toBroadcaster `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(to_broadcaster_sink)` `null::<(u32,)>()`
        
// toBroadcaster@(v%n)(v) :~ clientIn(v), numBroadcasterPartitions(n)
toBroadcasterBuffer(v%n, v) :- clientIn(v), numBroadcasterPartitions(n)
toBroadcasterBufferCount(dest, count(v)) :- toBroadcasterBuffer(dest, v)
toBroadcasterFull(dest) :- toBroadcasterBufferCount(dest, n), flushEveryN(n)
toBroadcaster@dest(v) :~ toBroadcasterBuffer(dest, v), toBroadcasterFull(dest)
toBroadcasterBuffer(dest, v) :+ toBroadcasterBuffer(dest, v), !toBroadcasterFull(dest)
    "#
    );

    df.run_async().await;
}

