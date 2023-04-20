use hydroflow::{util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
}};
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;

    let vote_to_vote_requester_source = ports
        .remove("vote_to_vote_requester")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let vote_to_participant = ports
        .remove("vote_to_participant")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;
    println!("participant_voter_ids: {:?}", vote_to_participant.keys.clone());
    let vote_to_participant_sink = vote_to_participant.into_sink();

    let (num_participant_voters, participant_voter_start_ids): (u32, Vec<u32>) = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();

    let mut df = datalog!(
        r#"
        ######################## relation definitions
# EDB
.input numParticipantVoters `repeat_iter([(num_participant_voters,),])` 
.input participantVoterStartIDs `repeat_iter(participant_voter_start_ids.clone()) -> map(|p| (p,))` # Assume = 0,n,2n,...,n*m, for n participants and m voter partitions

.async voteToVoteRequester `null::<(u32,u32,)>()` `source_stream(vote_to_vote_requester_source) -> map(|v| deserialize_from_bytes::<(u32,u32,)>(v.unwrap()).unwrap())`
.async voteToParticipant `map(|(node_id, v):(u32,(u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(vote_to_participant_sink)` `null::<(u32,u32,)>()`
######################## end relation definitions

voteToParticipant@(s+(tid%n))(tid, p) :~ voteToVoteRequester(tid, p), numParticipantVoters(n), participantVoterStartIDs(s)
        "#
    );

    df.run_async().await;
}