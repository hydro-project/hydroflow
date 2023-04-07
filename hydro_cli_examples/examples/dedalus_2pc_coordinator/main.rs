use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
};
use hydroflow::tokio_stream::wrappers::IntervalStream;
use hydroflow_datalog::datalog;
use tokio::time::{interval_at, Duration, Instant};

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

    let instruct_to_participant_sink = ports
        .remove("instruct_to_participant")
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

    let frequency = 1;
    let start = Instant::now() + Duration::from_secs(frequency);
    let periodic_source = IntervalStream::new(interval_at(start, Duration::from_secs(frequency)));

    let mut df = datalog!(
        r#" 
        .input clientIn `repeat_iter(vec![()]) -> map(|_| (context.current_tick() as u32,))`
.output clientOut `for_each(|(slot,payload):(u32,u32)| println!("completed {:?}: {:?}", slot, payload))`

.input periodic `source_stream(periodic_source) -> map(|_| ())`
.output throughputOut `for_each(|(num,):(u32,)| println!("completed {:?} entries", num))`

# EDBs
.input startSlot `repeat_iter([(0 as u32,),])`
.input participants `repeat_iter(peers.clone()) -> map(|p| (p,))`
.input success `repeat_iter([(true,),])`
.input reject `repeat_iter([(false,),])`
.input commitInstruct `repeat_iter([(true,),])`
.input rollbackInstruct `repeat_iter([(false,),])`

.async voteToParticipant `map(|(node_id, v):(u32,(u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(vote_to_participant_sink)` `null::<(u32,u32,)>()`
.async voteFromParticipant `null::<(u32,u32,bool,u32,)>()` `source_stream(vote_from_participant_source) -> map(|v| deserialize_from_bytes::<(u32,u32,bool,u32,)>(v.unwrap()).unwrap())`
.async instructToParticipant `map(|(node_id, v):(u32,(u32,u32,bool))| (node_id, serialize_to_bytes(v))) -> dest_sink(instruct_to_participant_sink)` `null::<(u32,u32,bool,)>()`
.async ackFromParticipant `null::<(u32,u32,u32,)>()` `source_stream(ack_from_participant_source) -> map(|v| deserialize_from_bytes::<(u32,u32,u32,)>(v.unwrap()).unwrap())`

# Phase 1a
nextSlot(s) :+ !nextSlot(s2), startSlot(s)
nextSlot(s+1) :+ nextSlot(s), ChosenPayload(payload)
nextSlot(s) :+ nextSlot(s), !ChosenPayload(payload)

ChosenPayload(choose(payload)) :- clientIn(payload)
voteToParticipant@addr(s, p) :~ participants(addr), ChosenPayload(p), nextSlot(s)


# Phase 1b, Phase 2a
unanimous(count(addr)) :- participants(addr)

AllVotes(s, payload, res, src) :+ AllVotes(s, payload, res, src), !committed(s, payload), !aborted(s, payload)
AllVotes(i, msg, res, l_from) :- voteFromParticipant(i, msg, res, l_from)

NumYesVotes(s, payload, count(src)) :- AllVotes(s, payload, res, src), success(res)
committed(s, payload) :- NumYesVotes(s, payload, num), unanimous(num)
instructToParticipant@addr(s, payload, commit) :~ committed(s, payload), participants(addr), commitInstruct(commit)

aborted(s, payload) :- AllVotes(s, payload, res, src), reject(res)
instructToParticipant@addr(s, payload, rollback) :~ aborted(s, payload), participants(addr), rollbackInstruct(rollback)


# Phase 2b
AllAcks(s, payload, src) :+ AllAcks(s, payload, src), !completed(s, payload)
AllAcks(s, payload, src) :- ackFromParticipant(s, payload, src)

NumAcks(s, payload, count(src)) :- AllAcks(s, payload, src)
completed(s, payload) :- NumAcks(s, payload, num), unanimous(num)
// clientOut(s, payload) :- completed(s, payload)

NumCompleted(count(s)) :- completed(s, payload)
totalCompleted(new) :+ !totalCompleted(prev), NumCompleted(new)
totalCompleted(prev) :+ totalCompleted(prev), !NumCompleted(new)
totalCompleted(prev + new) :+ totalCompleted(prev), NumCompleted(new)
throughputOut(num) :- totalCompleted(num), periodic()
    "#
    );

    df.run_async().await;
}
