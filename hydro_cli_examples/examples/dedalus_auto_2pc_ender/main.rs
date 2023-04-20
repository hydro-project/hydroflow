use hydroflow::{util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
}};
use hydroflow::tokio_stream::wrappers::IntervalStream;
use tokio::time::{interval_at, Duration, Instant};
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;

    let ack_from_participant_source = ports
        .remove("ack_from_participant")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let (my_id, num_participants): (u32, u32) = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();

    let frequency = 1;
    let start = Instant::now() + Duration::from_secs(frequency);
    let periodic_source = IntervalStream::new(interval_at(start, Duration::from_secs(frequency)));

    let mut df = datalog!(
        r#"
        ######################## relation definitions
# EDB
.input id `repeat_iter([(my_id,),])`
.input numParticipants `repeat_iter([(num_participants,),])`

.output clientOut `for_each(|(tid,payload):(u32,u32)| println!("completed {:?}: {:?}", tid, payload))`

.input periodic `source_stream(periodic_source) -> map(|_| ())`
.output throughputOut `for_each(|(id,num,):(u32,u32,)| println!("total_throughput,{:?},{:?}", id, num))`

.async ackFromParticipant `null::<(u32,u32,u32,)>()` `source_stream(ack_from_participant_source) -> map(|v| deserialize_from_bytes::<(u32,u32,u32,)>(v.unwrap()).unwrap())`
######################## end relation definitions

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
throughputOut(i, num) :- totalCompleted(num), periodic(), id(i)
        "#
    );

    df.run_async().await;
}