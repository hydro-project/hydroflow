use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
};
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let vote_to_participant_source = ports
        .remove("vote_to_participant")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let vote_from_participant_port = ports
        .remove("vote_from_participant")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;

    let peers = vote_from_participant_port.keys.clone();
    let vote_from_participant_sink = vote_from_participant_port.into_sink();

    let instruct_to_participant_source = ports
        .remove("instruct_to_participant")
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

    let my_id: Vec<u32> = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();
    println!("my_id: {:?}", my_id);
    println!("coordinator: {:?}", peers);

    let mut df = datalog!(
        r#"
        .input myID `repeat_iter(my_id.clone()) -> map(|p| (p,))`
        .input coordinator `repeat_iter(peers.clone()) -> map(|p| (p,))`
        .input verdict `repeat_iter([(true,),])`
        
        .async voteToParticipant `null::<(u32,u32,)>()` `source_stream(vote_to_participant_source) -> map(|x| deserialize_from_bytes::<(u32,u32,)>(x.unwrap()).unwrap())`
        .async voteFromParticipant `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(vote_from_participant_sink)` `null::<(u32,u32,)>()`
        .async instructToParticipant `null::<(u32,u32,bool,)>()` `source_stream(instruct_to_participant_source) -> map(|x| deserialize_from_bytes::<(u32,u32,bool,)>(x.unwrap()).unwrap())`
        .async ackFromParticipant `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(ack_from_participant_sink)` `null::<(u32,u32,u32,)>()`
            
        voteFromParticipant@addr(s, p, res, i) :~ voteToParticipant(s, p), coordinator(addr), myID(i), verdict(res)
        ackFromParticipant@addr(s, p, i) :~ instructToParticipant(s, p, b), coordinator(addr), myID(i) 
        "#
    );

    df.run_async().await;
}
