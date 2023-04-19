use std::path::Path;
use hydroflow::{util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
}, tokio_util::codec::{LinesCodec, Framed}};
use hydroflow_datalog::datalog;
use tokio::fs;
use tokio::fs::OpenOptions;

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

    let (my_id, log_directory, participant_log): (u32, String, String) = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();
    println!("my_id: {:?}", my_id);
    println!("coordinator: {:?}", peers);

    // logging
    fs::create_dir_all(log_directory.clone()).await.unwrap();
    let path = Path::new(".").join(log_directory).join(participant_log);
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .unwrap();
    file.set_len(0).await.unwrap();
    let file_sink = Framed::new(file, LinesCodec::new());

    let mut df = datalog!(
        r#"
        ######################## relation definitions
# EDB
.input myID `repeat_iter([(my_id,),])`
.input coordinator `repeat_iter(peers.clone()) -> map(|p| (p,))`

.async voteToParticipant `null::<(u64,u32,)>()` `source_stream(vote_to_participant_source) -> map(|x| deserialize_from_bytes::<(u64,u32,)>(x.unwrap()).unwrap())`
.async voteFromParticipant `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(vote_from_participant_sink)` `null::<(u64,u32,u32,)>()`
.async commitToParticipant `null::<(u64,u32,)>()` `source_stream(commit_to_participant_source) -> map(|x| deserialize_from_bytes::<(u64,u32,)>(x.unwrap()).unwrap())`
.async ackFromParticipant `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(ack_from_participant_sink)` `null::<(u64,u32,u32,)>()`

.output logVote `map(|(tid,p):(u64,u32)| format!("tid: {:?}, p: {:?}", tid, p)) -> dest_sink(file_sink)`
// .output logVoteComplete `for_each(|(tid,p,):(u64,u32,)| println!("logVoteComplete: tid: {:?}, p: {:?}", tid, p))`
######################## end relation definitions

logVote(tid, p) :- voteToParticipant(tid, p)
logVoteComplete(tid, p) :+ voteToParticipant(tid, p)
voteFromParticipant@addr(tid, p, i) :~ logVoteComplete(tid, p), coordinator(addr), myID(i)
ackFromParticipant@addr(tid, p, i) :~ commitToParticipant(tid, p), coordinator(addr), myID(i)
        "#
    );

    df.run_async().await;
}