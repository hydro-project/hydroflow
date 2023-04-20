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

    let vote_from_participant_sink = ports
        .remove("vote_from_participant")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await
        .into_sink();

    let (my_id, num_committers, log_directory, participant_log): (u32, u32, String, String) = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();

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
.input numCommitters `repeat_iter([(num_committers,),])`

.async voteToParticipant `null::<(u32,u32,)>()` `source_stream(vote_to_participant_source) -> map(|x| deserialize_from_bytes::<(u32,u32,)>(x.unwrap()).unwrap())`
.async voteFromParticipant `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(vote_from_participant_sink)` `null::<(u32,u32,u32,)>()`

.output logVote `map(|(tid,p):(u32,u32)| format!("tid: {:?}, p: {:?}", tid, p)) -> dest_sink(file_sink)`
# For some reason Hydroflow can't infer the type of logVoteComplete, so we define it manually:
.input logVoteComplete `null::<(u32,u32)>()`
######################## end relation definitions

logVote(tid, p) :- voteToParticipant(tid, p)
logVoteComplete(tid, p) :+ voteToParticipant(tid, p)
voteFromParticipant@(tid%n)(tid, p, i) :~ logVoteComplete(tid, p), myID(i), numCommitters(n)
        "#
    );

    df.run_async().await;
}