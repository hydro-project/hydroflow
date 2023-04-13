use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
};
use hydroflow::bytes::BytesMut;
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;

    let p1a_vote_source = ports
        .remove("p1a_vote")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let p1a_commit_sink = ports
        .remove("p1a_commit")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await
        .into_sink();

    let (my_id,num_acceptor_groups,acceptor_partitions):(u32,u32,Vec<u32>) = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();

    
    let mut df = datalog!(
        r#"
        ######################## relation definitions
# EDB
.input id `repeat_iter([(my_id,),])`
.input numAcceptorGroups `repeat_iter([(num_acceptor_groups,),])`
.input acceptorPartitions `repeat_iter(acceptor_partitions.clone()) -> map(|p| (p,))`

# Debug
.output p1aVoteOut `for_each(|(i, aid, pid, ballot_id, ballot_num):(u32,u32,u32,u32,u32,)| println!("coordinator {:?} received p1aVote from acceptor: [{:?},{:?},{:?},{:?}]]", i, aid, pid, ballot_id, ballot_num))`
.output p1aCommitOut `for_each(|(i, aid, o, pid, ballot_id, ballot_num):(u32,u32,u32,u32,u32,u32,)| println!("coordinator {:?} sent p1aVote to acceptor {:?}: [{:?},{:?},{:?},{:?}]]", i, aid, o, pid, ballot_id, ballot_num))`

# p1aVote: acceptorPartitionID, proposerID, ballotID, ballotNum
.async p1aVoteU `null::<(u32,u32,u32,u32,)>()` `source_stream(p1a_vote_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,u32,u32,)>(v.unwrap()).unwrap())`
# p1aCommit: order, proposerID, ballotID, ballotNum
.async p1aCommit `map(|(node_id, v):(u32,(u32,u32,u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(p1a_commit_sink)` `null::<(u32,u32,u32,u32)>()`
######################## end relation definitions


# Debug
// p1aVoteOut(i, aid, pid, ballotID, ballotNum) :- p1aVoteU(aid, pid, ballotID, ballotNum), id(i)
// p1aCommitOut(i, a, o, pid, ballotID, ballotNum) :- chosenP1aVote(pid, ballotID, ballotNum), nextOrder(o), acceptorPartitions(a), id(i)
// p1aCommitOut(i, a, 0, pid, ballotID, ballotNum) :- chosenP1aVote(pid, ballotID, ballotNum), !nextOrder(o), acceptorPartitions(a), id(i)


p1aVote(aid, pid, i, n) :- p1aVoteU(aid, pid, i, n)
p1aVote(aid, pid, i, n) :+ p1aVote(aid, pid, i, n), !chosenP1aVote(pid, i, n)


numP1aVotes(count(aid), pid, i, n) :- p1aVote(aid, pid, i, n)
committedP1aVotes(pid, i, n) :- numP1aVotes(c, pid, i, n), numAcceptorGroups(c)
chosenP1aVote(choose(pid), choose(i), choose(n)) :- committedP1aVotes(pid, i, n)
p1aCommit@a(o, pid, i, n) :~ chosenP1aVote(pid, i, n), nextOrder(o), acceptorPartitions(a)
p1aCommit@a(0, pid, i, n) :~ chosenP1aVote(pid, i, n), !nextOrder(o), acceptorPartitions(a)


nextOrder(1) :+ chosenP1aVote(_, _, _), !nextOrder(o)
nextOrder(o) :+ !chosenP1aVote(_, _, _), nextOrder(o)
nextOrder(o+1) :+ chosenP1aVote(_, _, _), nextOrder(o)
"#
    );

    df.run_async().await;
}