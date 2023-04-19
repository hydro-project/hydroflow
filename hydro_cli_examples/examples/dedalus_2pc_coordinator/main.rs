use std::path::Path;
use hydroflow::{util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
}, tokio_util::codec::{LinesCodec, Framed}};
use hydroflow::tokio_stream::wrappers::IntervalStream;
use hydroflow_datalog::datalog;
use tokio::time::{interval_at, Duration, Instant};
use tokio::fs;
use tokio::fs::OpenOptions;


#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let vote_to_participant_port = ports
        .remove("vote_to_participant")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;

    let peers = vote_to_participant_port.keys.clone();
    println!("peers: {:?}", peers);
    let vote_to_participant_sink = vote_to_participant_port.into_sink();

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

    let ack_from_participant_source = ports
        .remove("ack_from_participant")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let (num_participants, log_directory, coordinator_log): (u32, String, String) = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();

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

    let frequency = 1;
    let start = Instant::now() + Duration::from_secs(frequency);
    let periodic_source = IntervalStream::new(interval_at(start, Duration::from_secs(frequency)));

    let mut df = datalog!(
        r#" 
        ######################## relation definitions
# EDB
.input participants `repeat_iter(peers.clone()) -> map(|p| (p,))`
.input numParticipants `repeat_iter([(num_participants,),])`

.input clientIn `repeat_iter(vec![()]) -> map(|_| (context.current_tick() as u64, context.current_tick() as u32))`
.output clientOut `for_each(|(tid,payload):(u64,u32)| println!("completed {:?}: {:?}", tid, payload))`

.input periodic `source_stream(periodic_source) -> map(|_| ())`
.output throughputOut `for_each(|(num,):(u32,)| println!("total_throughput,{:?}", num))`

.async voteToParticipant `map(|(node_id, v):(u32,(u64,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(vote_to_participant_sink)` `null::<(u64,u32,)>()`
.async voteFromParticipant `null::<(u64,u32,u32,)>()` `source_stream(vote_from_participant_source) -> map(|v| deserialize_from_bytes::<(u64,u32,u32,)>(v.unwrap()).unwrap())`
.async commitToParticipant `map(|(node_id, v):(u32,(u64,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(commit_to_participant_sink)` `null::<(u64,u32,)>()`
.async ackFromParticipant `null::<(u64,u32,u32,)>()` `source_stream(ack_from_participant_source) -> map(|v| deserialize_from_bytes::<(u64,u32,u32,)>(v.unwrap()).unwrap())`

.output logCommit `map(|(tid,p):(u64,u32)| format!("tid: {:?}, p: {:?}", tid, p)) -> dest_sink(file_sink)`
######################## end relation definitions

# Phase 1a
voteToParticipant@addr(tid, p) :~ participants(addr), clientIn(tid, p)

# Phase 1b, Phase 2a
AllVotes(tid, payload, src) :+ AllVotes(tid, payload, src), !committed(tid, _)
AllVotes(tid, payload, src) :- voteFromParticipant(tid, payload, src)

NumYesVotes(tid, count(src)) :- AllVotes(tid, payload, src)
committed(tid, payload) :- NumYesVotes(tid, num), AllVotes(tid, payload, src), numParticipants(num) 
logCommit(tid, payload) :- committed(tid, payload)
logCommitComplete(tid, payload) :+ committed(tid, payload)
commitToParticipant@addr(tid, payload) :~ logCommitComplete(tid, payload), participants(addr)

# Phase 2b
AllAcks(tid, payload, src) :+ AllAcks(tid, payload, src), !completed(tid, _)
AllAcks(tid, payload, src) :- ackFromParticipant(tid, payload, src)

NumAcks(tid, count(src)) :- AllAcks(tid, payload, src)
completed(tid, payload) :- NumAcks(tid, num), AllAcks(tid, payload, src), numParticipants(num)
// clientOut(tid, payload) :- completed(tid, payload)

NumCompleted(count(tid)) :- completed(tid, payload)
totalCompleted(new) :+ !totalCompleted(prev), NumCompleted(new)
totalCompleted(prev) :+ totalCompleted(prev), !NumCompleted(new)
totalCompleted(prev + new) :+ totalCompleted(prev), NumCompleted(new)
throughputOut(num) :- totalCompleted(num), periodic()
    "#
    );

    df.run_async().await;
}