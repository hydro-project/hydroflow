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

    let vote_from_participant_source = ports
        .remove("vote_from_participant")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let commit_to_participant_sink = ports
        .remove("commit_to_participant")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await
        .into_sink();

    let (num_participants, num_participant_ackers, participant_acker_start_ids, log_directory, coordinator_log): (u32, u32, Vec<u32>, String, String) = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();

    // logging
    fs::create_dir_all(log_directory.clone()).await.unwrap();
    let path = Path::new(".").join(log_directory).join(coordinator_log);
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
.input numParticipants `repeat_iter([(num_participants,),])`
.input numParticipantACKers `repeat_iter([(num_participant_ackers,),])`
.input participantACKerStartIDs `repeat_iter(participant_acker_start_ids.clone()) -> map(|p| (p,))` # Assume = 0,n,2n,...,n*m, for n participants and m acker partitions

.async voteFromParticipant `null::<(u32,u32,u32,)>()` `source_stream(vote_from_participant_source) -> map(|v| deserialize_from_bytes::<(u32,u32,u32,)>(v.unwrap()).unwrap())`
.async commitToParticipant `map(|(node_id, v):(u32,(u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(commit_to_participant_sink)` `null::<(u32,u32,)>()`

.output logCommit `map(|(tid,p):(u32,u32)| format!("tid: {:?}, p: {:?}", tid, p)) -> dest_sink(file_sink)`
# For some reason Hydroflow can't infer the type of logCommitComplete, so we define it manually:
.input logCommitComplete `null::<(u32,u32)>()`
######################## end relation definitions

# Phase 1b, Phase 2a
AllVotes(tid, payload, src) :+ AllVotes(tid, payload, src), !committed(tid, _)
AllVotes(tid, payload, src) :- voteFromParticipant(tid, payload, src)

NumYesVotes(tid, count(src)) :- AllVotes(tid, payload, src)
committed(tid, payload) :- NumYesVotes(tid, num), AllVotes(tid, payload, src), numParticipants(num) 
logCommit(tid, payload) :- committed(tid, payload)
logCommitComplete(tid, payload) :+ committed(tid, payload)
commitToParticipant@(s+(tid%n))(tid, payload) :~ logCommitComplete(tid, payload), numParticipantACKers(n), participantACKerStartIDs(s) 
        "#
    );

    df.run_async().await;
}