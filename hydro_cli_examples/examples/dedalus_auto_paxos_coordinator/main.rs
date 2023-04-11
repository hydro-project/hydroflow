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

    let p1a_commit = ports
        .remove("p1a_commit")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;

    let acceptors = p1a_commit.keys.clone();
    println!("acceptors: {:?}", acceptors);
    let p1a_commit_sink = p1a_commit.into_sink();

    let num_acceptor_groups:Vec<u32> = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();

    
    let mut df = datalog!(
        r#"
        .input numAcceptorGroups `repeat_iter(num_acceptor_groups.clone()) -> map(|p| (p,))`
        .input acceptors `repeat_iter(acceptors.clone()) -> map(|p| (p,))`
        
        # p1aVote: acceptorPartitionID, proposerID, ballotID, ballotNum
        .async p1aVoteU `null::<(u32,u32,u32,u32,)>()` `source_stream(p1a_vote_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,u32,u32,)>(v.unwrap()).unwrap())`
        # p1aCommit: order, proposerID, ballotID, ballotNum
        .async p1aCommit `map(|(node_id, v):(u32,(u32,u32,u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(p1a_commit_sink)` `null::<(u32,u32,u32,u32)>()`
        
        
        p1aVote(aid, pid, i, n) :- p1aVoteU(aid, pid, i, n)
        p1aVote(aid, pid, i, n) :+ p1aVote(aid, pid, i, n)
        
        
        numP1aVotes(count(aid), pid, i, n) :- p1aVote(aid, pid, i, n)
        committedP1aVotes(pid, i, n) :- numP1aVotes(c, pid, i, n), numAcceptorGroups(c)
        chosenP1aVote(choose(pid), choose(i), choose(n)) :- committedP1aVotes(pid, i, n)
        p1aCommit@a(o, pid, i, n) :~ chosenP1aVote(pid, i, n), nextOrder(o), acceptors(a)
        p1aCommit@a(0, pid, i, n) :~ chosenP1aVote(pid, i, n), !nextOrder(o), acceptors(a)
        
        
        nextOrder(1) :+ chosenP1aVote(_, _, _), !nextOrder(o)
        nextOrder(o) :+ !chosenP1aVote(_, _, _), nextOrder(o)
        nextOrder(o+1) :+ chosenP1aVote(_, _, _), nextOrder(o)
"#
    );

    df.run_async().await;
}