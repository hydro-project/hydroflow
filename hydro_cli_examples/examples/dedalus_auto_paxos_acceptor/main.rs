use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
};
use hydroflow::bytes::BytesMut;
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let p1a_source = ports
        .remove("p1a")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let p1b_sink = ports
        .remove("p1b")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await
        .into_sink();

    let p1b_log_sink = ports
        .remove("p1b_log")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await
        .into_sink();

    let p2a_source = ports
        .remove("p2a")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let p2b_ports = ports
        .remove("p2b")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;
    let p2b_proxy_leaders = p2b_ports.keys.clone();
    println!("p2b_proxy_leaders: {:?}", p2b_proxy_leaders);
    let p2b_sink = p2b_ports.into_sink();

    let p1a_vote_ports = ports
        .remove("p1a_vote")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;
    let coordinators = p1a_vote_ports.keys.clone();
    println!("coordinators: {:?}", coordinators);
    let p1a_vote_sink = p1a_vote_ports.into_sink();

    let p1a_commit_source = ports
        .remove("p1a_commit")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let (acceptor_id, partition_id, coordinator, num_p2b_proxy_leaders):(u32,u32,u32,u32) = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();

    let mut df = datalog!(
        r#"
        ######################## relation definitions
        # EDB
        .input acceptorID `repeat_iter([(acceptor_id,),])` # Shared by all partitions of the same acceptor.
        .input partitionID `repeat_iter([(partition_id,),])`
        .input coordinator `repeat_iter([(coordinator,),])`
        .input numP2bProxyLeaders `repeat_iter([(num_p2b_proxy_leaders,),])` # ID scheme: Assuming num_p2b_proxy_leaders = n (per proposer). Proposer i has proxy leaders from i*n to (i+1)*n-1
        
        # Debug
        .output p1aOut `for_each(|(a,pid,id,num):(u32,u32,u32,u32,)| println!("acceptor {:?} received p1a: [{:?},{:?},{:?}]", a, pid, id, num))`
        .output p1bOut `for_each(|(pid,a,log_size,id,num,max_id,max_num):(u32,u32,u32,u32,u32,u32,u32,)| println!("acceptor {:?} sent p1b to {:?}: [{:?},{:?},{:?},{:?},{:?},{:?}]", a, pid, a, log_size, id, num, max_id, max_num))`
        .output p1bLogOut `for_each(|(pid,a,payload,slot,payload_id,payload_num,id,num):(u32,u32,u32,u32,u32,u32,u32,u32,)| println!("acceptor {:?} sent p1bLog to {:?}: [{:?},{:?},{:?},{:?},{:?},{:?},{:?}]", a, pid, a, payload, slot, payload_id, payload_num, id, num))`
        .output p2aOut `for_each(|(a,pid,payload,slot,id,num):(u32,u32,u32,u32,u32,u32,)| println!("acceptor {:?} received p2a: [{:?},{:?},{:?},{:?},{:?}]", a, pid, payload, slot, id, num))`
        .output p2bOut `for_each(|(pid,a,payload,slot,id,num,max_id,max_num):(u32,u32,u32,u32,u32,u32,u32,u32,)| println!("acceptor {:?} sent p2b to {:?}: [{:?},{:?},{:?},{:?},{:?},{:?},{:?}]]", a, pid, a, payload, slot, id, num, max_id, max_num))`
        # For some reason Hydroflow can't infer the type of p2aSealed, so we define it manually:
        .input p2aSealed `null::<(u32,u32,u32,u32,u32)>()`
        
        # p1a: proposerID, ballotID, ballotNum
        .async p1aU `null::<(u32,u32,u32,)>()` `source_stream(p1a_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,u32,)>(v.unwrap()).unwrap())`
        # p1b: partitionID, acceptorID, logSize, ballotID, ballotNum, maxBallotID, maxBallotNum
        .async p1b `map(|(node_id, v):(u32,(u32,u32,u32,u32,u32,u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(p1b_sink)` `null::<(u32,u32,u32,u32,u32,u32,u32)>()`
        # p1bLog: partitionID, acceptorID, payload, slot, payloadBallotID, payloadBallotNum, ballotID, ballotNum
        .async p1bLog `map(|(node_id, v):(u32,(u32,u32,u32,u32,u32,u32,u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(p1b_log_sink)` `null::<(u32,u32,u32,u32,u32,u32,u32,u32)>()`
        # p2a: proposerID, payload, slot, ballotID, ballotNum
        .async p2aU `null::<(u32,u32,u32,u32,u32,)>()` `source_stream(p2a_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,u32,u32,u32,)>(v.unwrap()).unwrap())`
        # p2b: acceptorID, payload, slot, ballotID, ballotNum, maxBallotID, maxBallotNum
        .async p2b `map(|(node_id, v):(u32,(u32,u32,u32,u32,u32,u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(p2b_sink)` `null::<(u32,u32,u32,u32,u32,u32,u32)>()`
        
        # p1aVote: acceptorPartitionID, proposerID, ballotID, ballotNum
        .async p1aVote `map(|(node_id, v):(u32,(u32,u32,u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(p1a_vote_sink)` `null::<(u32,u32,u32,u32)>()`
        # p1aCommit: order, proposerID, ballotID, ballotNum
        .async p1aCommitU `null::<(u32,u32,u32,u32,)>()` `source_stream(p1a_commit_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,u32,u32,)>(v.unwrap()).unwrap())`
        ######################## end relation definitions
        
        
        .persist ballots
        .persist log
        
        
        # Debug
        // p1aOut(a, pid, id, num) :- p1a(pid, id, num), id(a)
        // p1bOut(pid, i, size, ballotID, ballotNum, maxBallotID, maxBallotNum) :- p1a(pid, ballotID, ballotNum), id(i), LogSize(size), MaxBallot(maxBallotID, maxBallotNum)
        // p1bOut(pid, i, 0, ballotID, ballotNum, maxBallotID, maxBallotNum) :- p1a(pid, ballotID, ballotNum), id(i), !LogSize(size), MaxBallot(maxBallotID, maxBallotNum)
        // p1bLogOut(pid, i, payload, slot, payloadBallotID, payloadBallotNum, ballotID, ballotNum) :- p1a(pid, ballotID, ballotNum), id(i), log(payload, slot, payloadBallotID, payloadBallotNum), LogEntryMaxBallot(slot, payloadBallotID, payloadBallotNum)
        // p2aOut(a, pid, payload, slot, id, num) :- p2a(pid, payload, slot, id, num), id(a)
        // p2bOut(pid, i, payload, slot, ballotID, ballotNum, maxBallotID, maxBallotNum) :- p2a(pid, payload, slot, ballotID, ballotNum), id(i), MaxBallot(maxBallotID, maxBallotNum)
        
        
        ######################## reply to p1a 
        ballots(id, num) :- p1aSealed(_, pid, id, num)
MaxBallotNum(max(num)) :- ballots(id, num) 
MaxBallot(max(id), num) :- MaxBallotNum(num), ballots(id, num)
LogSize(count(slot)) :- log(p, slot, ballotID, ballotNum), p1aSealed(_, _, _, _)
p1b@pid(p, i, size, ballotID, ballotNum, maxBallotID, maxBallotNum) :~ p1aSealed(_, pid, ballotID, ballotNum), acceptorID(i), partitionID(p), LogSize(size), MaxBallot(maxBallotID, maxBallotNum)
p1b@pid(p, i, 0, ballotID, ballotNum, maxBallotID, maxBallotNum) :~ p1aSealed(_, pid, ballotID, ballotNum), acceptorID(i), partitionID(p), !LogSize(size), MaxBallot(maxBallotID, maxBallotNum)

LogEntryMaxBallotNum(slot, max(ballotNum)) :- log(p, slot, ballotID, ballotNum), p1aSealed(_, _, _, _)
LogEntryMaxBallot(slot, max(ballotID), ballotNum) :- LogEntryMaxBallotNum(slot, ballotNum), log(p, slot, ballotID, ballotNum), p1aSealed(_, _, _, _)

# send back entire log 
p1bLog@pid(p, i, payload, slot, payloadBallotID, payloadBallotNum, ballotID, ballotNum) :~ p1aSealed(_, pid, ballotID, ballotNum), acceptorID(i), partitionID(p), log(payload, slot, payloadBallotID, payloadBallotNum), LogEntryMaxBallot(slot, payloadBallotID, payloadBallotNum)
        ######################## end reply to p1a 
        
        
        
        ######################## reply to p2a
        log(payload, slot, ballotID, ballotNum) :- p2aSealed(pid, payload, slot, ballotID, ballotNum), MaxBallot(ballotID, ballotNum)
        p2b@((pid*n)+(slot%n))(i, payload, slot, ballotID, ballotNum, maxBallotID, maxBallotNum) :~ p2aSealed(pid, payload, slot, ballotID, ballotNum), acceptorID(i), MaxBallot(maxBallotID, maxBallotNum), numP2bProxyLeaders(n)
        ######################## end reply to p2a
        
        
        
        
        ######################## partial partitioning
        processedI(i) :+ processedI(i)
        maxProcessedI(max(i)) :- processedI(i)
        maxReceivedI(max(i)) :- receivedI(i)
        unfreeze() :- maxReceivedI(i), maxProcessedI(i), !outstandingVote()
        
        p1a(pid, id, num) :- p1aU(pid, id, num)
        p1a(pid, id, num) :+ p1a(pid, id, num)
        p1aVote@c(i, pid, id, num) :~ p1a(pid, id, num), partitionID(i), coordinator(c)
        
        p1aCommit(i, pid, id, num) :- p1aCommitU(i, pid, id, num)
        p1aCommit(i, pid, id, num) :+ p1aCommit(i, pid, id, num)
        receivedI(i) :- p1aCommit(i, pid, id, num)
        nextToProcess(i+1) :- maxProcessedI(i)
        p1aSealed(i, pid, id, num) :- nextToProcess(i), p1aCommit(i, pid, id, num)
        p1aSealed(i, pid, id, num) :- !nextToProcess(i), p1aCommit(i, pid, id, num), (i == 0)
        processedI(i) :+ p1aSealed(i, pid, id, num)
        outstandingVote() :- p1a(pid, id, num), !p1aCommit(i, pid, id, num)
        
        p2a(pid, payload, slot, ballotID, ballotNum) :- p2aU(pid, payload, slot, ballotID, ballotNum)
        p2a(pid, payload, slot, ballotID, ballotNum) :+ p2a(pid, payload, slot, ballotID, ballotNum), !unfreeze()
        p2aSealed(pid, payload, slot, ballotID, ballotNum) :- p2a(pid, payload, slot, ballotID, ballotNum), unfreeze()
        ######################## end partial partitioning
        "#
       );

    df.run_async().await;
}

