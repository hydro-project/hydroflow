use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
};
use hydroflow::bytes::BytesMut;
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let p2a_source = ports
        .remove("p2a_to_proxy")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let p2a_sink = ports
        .remove("p2a_from_proxy")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await
        .into_sink();

    let (my_id, acceptor_start_ids, num_acceptor_groups):(u32, Vec<u32>, u32) = 
        serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();

    
    let mut df = datalog!(
        r#"
        ######################## relation definitions
# EDB
.input id `repeat_iter([(my_id,),])`
.input acceptorStartIDs `repeat_iter(acceptor_start_ids.clone()) -> map(|p| (p,))` # Assume = 0,n,2n,...,n*m, for n acceptors and m partitions
.input numAcceptorGroups `repeat_iter([(num_acceptor_groups,),])` 

# Debug
.output p2aOut `for_each(|(i,pid,payload,slot,id,num):(u32,u32,u32,u32,u32,u32,)| println!("p2aProxyLeader {:?} received p2a from proposer: [{:?},{:?},{:?},{:?},{:?}]", i, pid, payload, slot, id, num))`
.output p2aBroadcastOut `for_each(|(i,a,pid,payload,slot,id,num):(u32,u32,u32,u32,u32,u32,u32)| println!("p2aProxyLeader {:?} sent p2a to acceptor {:?}: [{:?},{:?},{:?},{:?},{:?}]", i, a, pid, payload, slot, id, num))`

# p2a: proposerID, payload, slot, ballotID, ballotNum
.async p2aIn `null::<(u32,u32,u32,u32,u32,)>()` `source_stream(p2a_source) -> map(|v: Result<BytesMut, _>| deserialize_from_bytes::<(u32,u32,u32,u32,u32,)>(v.unwrap()).unwrap())`
.async p2aBroadcast `map(|(node_id, v):(u32,(u32,u32,u32,u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(p2a_sink)` `null::<(u32,u32,u32,u32,u32)>()` 
# p2b: acceptorID, payload, slot, ballotID, ballotNum, maxBallotID, maxBallotNum
######################## end relation definitions

# Debug
// p2aOut(i, pid, payload, slot, id, num) :- p2aIn(pid, payload, slot, id, num), id(i)
// p2aBroadcastOut(i, aid+(slot%n), pid, payload, slot, id, num) :- p2aIn(pid, payload, slot, id, num), numAcceptorGroups(n), acceptorStartIDs(aid), id(i)
p2aBroadcast@(aid+(slot%n))(pid, payload, slot, id, num) :~ p2aIn(pid, payload, slot, id, num), numAcceptorGroups(n), acceptorStartIDs(aid)
"#
    );

    df.run_async().await;
}