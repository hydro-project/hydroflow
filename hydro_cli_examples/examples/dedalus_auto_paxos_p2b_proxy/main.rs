use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
};
use hydroflow::bytes::BytesMut;
use hydroflow::tokio_stream::wrappers::IntervalStream;
use hydroflow_datalog::datalog;
use tokio::time::{interval_at, Duration, Instant};

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;

    let p2b_source = ports
        .remove("p2b")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let p2b_to_proposer_sink = ports
        .remove("p2b_to_proposer")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await
        .into_sink();

    let inputs_sink = ports
        .remove("inputs")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await
        .into_sink();

    let (my_id, f, acceptor_start_ids, num_acceptor_groups, proposer):(u32, u32, Vec<u32>, u32, u32) = 
        serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();
    let periodic_source = periodic(1);

    
    let mut df = datalog!(
        r#"
        ######################## relation definitions
# EDB
.input id `repeat_iter([(my_id,),])`
.input quorum `repeat_iter([(f+1,),])`
.input acceptorStartIDs `repeat_iter(acceptor_start_ids.clone()) -> map(|p| (p,))` # Assume = 0,n,2n,...,n*m, for n acceptors and m partitions
.input numAcceptorGroups `repeat_iter([(num_acceptor_groups,),])`
.input proposer `repeat_iter([(proposer,),])` # The proposer this proxy leader was decoupled from
.input tick `repeat_iter(vec![()]) -> map(|_| (context.current_tick() as u32,))`

# Debug
.output p2bOut `for_each(|(i,a,payload,slot,id,num,max_id,max_num):(u32,u32,u32,u32,u32,u32,u32,u32,)| println!("p2bProxyLeader {:?} received p2b from acceptor: [{:?},{:?},{:?},{:?},{:?},{:?},{:?}]]", i, a, payload, slot, id, num, max_id, max_num))`
.output p2bToProposerOut `for_each(|(i,pid,max_id,max_num,t1):(u32,u32,u32,u32,u32,)| println!("p2bProxyLeader {:?} sent p2b to proposer {:?}: [{:?},{:?},{:?}]]", i, pid, max_id, max_num, t1))`
.output p2bToProposerOut `for_each(|(i,pid,n,t1,prev_t):(u32,u32,u32,u32,u32,)| println!("p2bProxyLeader {:?} sent inputs to proposer {:?}: [{:?},{:?},{:?}]]", i, pid, n, t1, prev_t))`
.input periodic `source_stream(periodic_source) -> map(|_| ())`
.output throughputOut `for_each(|(id,num,):(u32,u32,)| println!("proxy leader {:?}: {:?}", id, num))`

# p2b: acceptorID, payload, slot, ballotID, ballotNum, maxBallotID, maxBallotNum
.async p2bU `null::<(u32,u32,u32,u32,u32,u32,u32)>()` `source_stream(p2b_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,u32,u32,u32,u32,u32,)>(v.unwrap()).unwrap())`
# p2bToProposer: maxBallotID, maxBallotNum, t1
.async p2bToProposer `map(|(node_id, v):(u32,(u32,u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(p2b_to_proposer_sink)` `null::<(u32,u32,u32)>()` 
# inputs: n, t1, prevT
.async inputs `map(|(node_id, v):(u32,(u32,u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(inputs_sink)` `null::<(u32,u32,u32)>()` 
######################## end relation definitions


p2b(a, p, s, i, n, mi, mn) :- p2bU(a, p, s, i, n, mi, mn)
p2b(a, p, s, i, n, mi, mn) :+ p2b(a, p, s, i, n, mi, mn), !commit(p2, s) # drop all p2bs if slot s is committed


# Debug
// p2bOut(i, a, payload, slot, id, num, maxID, maxNum) :- id(i), p2bU(a, payload, slot, id, num, maxID, maxNum)
// p2bToProposerOut(i, pid, mi, mn, t1) :- p2bNewBallot(mi, mn), tick(t1), proposer(pid), id(i)
// inputsOut(i, pid, n, t1, prevT) :- batchSize(n), tick(t1), batchTimes(prevT), proposer(pid), id(i)
// inputsOut(i, pid, n, t1, 0) :- batchSize(n), tick(t1), !batchTimes(prevT), proposer(pid), id(i)
throughputOut(i, num) :- totalCommitted(num), periodic(), id(i)


######################## p2bs with asymmetric decoupling
p2bUniqueBallot(mi, mn) :+ p2bU(a, p, s, i, n, mi, mn)
p2bUniqueBallot(mi, mn) :+ p2bUniqueBallot(mi, mn)
p2bNewBallot(mi, mn) :- p2bU(a, p, s, i, n, mi, mn), !p2bUniqueBallot(mi, mn)
p2bToProposer@pid(mi, mn, t1) :~ p2bNewBallot(mi, mn), tick(t1), proposer(pid)
p2bUCount(count(*)) :- p2bNewBallot(mi, mn)
batchSize(n) :- p2bUCount(n)
batchTimes(t1) :+ batchSize(n), tick(t1) # Since there's only 1 r to be batched, (n != 0) is implied
batchTimes(t1) :+ !batchSize(n), batchTimes(t1) # Persist if no batch
inputs@pid(n, t1, prevT) :~ batchSize(n), tick(t1), batchTimes(prevT), proposer(pid)
inputs@pid(n, t1, 0) :~ batchSize(n), tick(t1), !batchTimes(prevT), proposer(pid)
######################## end p2bs with asymmetric decoupling


CountMatchingP2bs(payload, slot, count(acceptorID), i, num) :- p2b(acceptorID, payload, slot, i, num, payloadBallotID, payloadBallotNum)
commit(payload, slot) :- CountMatchingP2bs(payload, slot, c, i, num), quorum(size), (c >= size)
// clientOut(payload, slot) :- commit(payload, slot)
NumCommits(count(slot)) :- commit(payload, slot)

totalCommitted(new) :+ !totalCommitted(prev), NumCommits(new)
totalCommitted(prev) :+ totalCommitted(prev), !NumCommits(new)
totalCommitted(prev + new) :+ totalCommitted(prev), NumCommits(new)

"#
    );
    df.run_async().await;
}

fn periodic(timeout: u32) -> IntervalStream {
    let start = Instant::now() + Duration::from_secs(timeout.into());
    IntervalStream::new(interval_at(start, Duration::from_secs(timeout.into())))
}