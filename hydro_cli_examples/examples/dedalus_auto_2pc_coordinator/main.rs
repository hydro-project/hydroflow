use hydroflow::{util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
}};
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let vote_to_vote_requester = ports
        .remove("vote_to_vote_requester")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;
    println!("vote requesters: {:?}", vote_to_vote_requester.keys.clone());
    let vote_to_vote_requester_sink = vote_to_vote_requester.into_sink();

    let num_vote_requester_partitions: Vec<u32> = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();
    println!("num vote requesters: {:?}", num_vote_requester_partitions.clone());

    let mut df = datalog!(
        r#"
        ######################## relation definitions
        # EDB
        .input clientIn `repeat_iter_external(vec![()]) -> map(|_| (context.current_tick() as u32, context.current_tick() as u32))`
        .input numVoteRequesters `repeat_iter(num_vote_requester_partitions.clone()) -> map(|p| (p,))`

        .async voteToVoteRequester `map(|(node_id, v):(u32,(u32,u32))| (node_id, serialize_to_bytes(v))) -> dest_sink(vote_to_vote_requester_sink)` `null::<(u32,u32,)>()`
        ######################## end relation definitions
        
        # Phase 1a
        voteToVoteRequester@(tid%n)(tid, p) :~ clientIn(tid, p), numVoteRequesters(n)
        "#
    );

    df.run_async().await;
}