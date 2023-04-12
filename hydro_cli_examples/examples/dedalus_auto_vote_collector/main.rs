use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedSource},
    deserialize_from_bytes,
};
use hydroflow::tokio_stream::wrappers::IntervalStream;
use hydroflow_datalog::datalog;
use tokio::time::{interval_at, Duration, Instant};

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let from_participant_source = ports
        .remove("from_participant")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let (my_id, num_participants): (u32,u32) = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();

    let frequency = 1;
    let start = Instant::now() + Duration::from_secs(frequency);
    let periodic_source = IntervalStream::new(interval_at(start, Duration::from_secs(frequency)));

    let mut df = datalog!(
        r#"
        .input id `repeat_iter([(my_id,),])`
.output stdout `for_each(|s:(u32,)| println!("committed: {:?}", s))`
.input numParticipants `repeat_iter([(num_participants,),])`

.input periodic `source_stream(periodic_source) -> map(|_| ())`
.output throughputOut `for_each(|(i,num,):(u32,u32,)| println!("throughput,{:?},{:?}", i, num))`

.async voteFromParticipant `null::<(u32,u32,)>()` `source_stream(from_participant_source) -> map(|v| deserialize_from_bytes::<(u32,u32,)>(v.unwrap()).unwrap())`
        
allVotes(l, v) :- voteFromParticipant(l, v)
allVotes(l, v) :+ allVotes(l, v), !committed(v)
voteCounts(count(l), v) :- allVotes(l, v)
committed(v) :- voteCounts(n, v), numParticipants(n)
// stdout(v) :- committed(v)

NumCommits(count(v)) :- committed(v)
totalCommitted(new) :+ !totalCommitted(prev), NumCommits(new)
totalCommitted(prev) :+ totalCommitted(prev), !NumCommits(new)
totalCommitted(prev + new) :+ totalCommitted(prev), NumCommits(new)
throughputOut(i,num) :- totalCommitted(num), periodic(), id(i)
    "#
    );

    df.run_async().await;
}

