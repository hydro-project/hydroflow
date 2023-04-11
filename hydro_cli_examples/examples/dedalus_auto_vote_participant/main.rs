use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
};
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let to_participant_source = ports
        .remove("to_participant")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let from_participant_port = ports
        .remove("from_participant")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;

    let peers = from_participant_port.keys.clone();
    println!("collectors: {:?}", peers);
    let from_participant_sink = from_participant_port.into_sink();

    let (my_id, num_collector_partitions): (u32,u32) = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();
    println!("my_id: {:?}", my_id);

    let mut df = datalog!(
        r#"
        .input myID `repeat_iter([(my_id,),])`
.input numCollectorPartitions `repeat_iter([(num_collector_partitions,),])` # Assume id = 0,1,2...

.async voteToParticipant `null::<(u32,)>()` `source_stream(to_participant_source) -> map(|x| deserialize_from_bytes::<(u32,)>(x.unwrap()).unwrap())`
.async voteFromParticipant `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(from_participant_sink)` `null::<(u32,u32,)>()`
// .output out `for_each(|(coll,id,v):(u32,u32,u32)| println!("coll,id,v: [{:?},{:?},{:?}]", coll, id, v))`
            
out(v%n, i, v) :- voteToParticipant(v), numCollectorPartitions(n), myID(i)
voteFromParticipant@(v%n)(i, v) :~ voteToParticipant(v), numCollectorPartitions(n), myID(i)
        "#
    );

    df.run_async().await;
}
