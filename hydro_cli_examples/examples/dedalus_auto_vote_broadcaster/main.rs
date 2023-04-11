use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
};
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let to_broadcaster_source = ports
        .remove("to_broadcaster")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let to_participant_port = ports
        .remove("to_participant")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;

    let participants = to_participant_port.keys.clone();
    println!("participants: {:?}", participants);
    let to_participant_sink = to_participant_port.into_sink();

    let (participant_start_ids, num_participant_partitions): (Vec<u32>,u32) = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();
    println!("participant start ids: {:?}", participant_start_ids);

    let mut df = datalog!(
        r#"
        .input participantStartIDs `repeat_iter(participant_start_ids.clone()) -> map(|p| (p,))` # Assume = 0,n,2n,...,n*m, for n participants and m partitions        
        .input numParticipantPartitions `repeat_iter([(num_participant_partitions,),])`
        .async toBroadcaster `null::<(u32,)>()` `source_stream(to_broadcaster_source) -> map(|x| deserialize_from_bytes::<(u32,)>(x.unwrap()).unwrap())`
        .async voteToParticipants `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(to_participant_sink)` `null::<(u32,)>()`  

        voteToParticipants@(p+(v%m))(v) :~ toBroadcaster(v), participantStartIDs(p), numParticipantPartitions(m)
"#
    );

    df.run_async().await;
}
