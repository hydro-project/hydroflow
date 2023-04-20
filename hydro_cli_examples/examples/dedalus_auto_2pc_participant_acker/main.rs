use hydroflow::{util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
}};
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;

    let commit_to_participant_source = ports
        .remove("commit_to_participant")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let ack_from_participant_sink = ports
        .remove("ack_from_participant")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await
        .into_sink();

    let (my_id, num_enders): (u32, u32) = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();

    let mut df = datalog!(
        r#"
        ######################## relation definitions
# EDB
.input myID `repeat_iter([(my_id,),])`
.input numEnders `repeat_iter([(num_enders,),])`

.async commitToParticipant `null::<(u32,u32,)>()` `source_stream(commit_to_participant_source) -> map(|x| deserialize_from_bytes::<(u32,u32,)>(x.unwrap()).unwrap())`
.async ackFromParticipant `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(ack_from_participant_sink)` `null::<(u32,u32,u32,)>()`

######################## end relation definitions

ackFromParticipant@(tid%n)(tid, p, i) :~ commitToParticipant(tid, p), myID(i), numEnders(n)
        "#
    );

    df.run_async().await;
}