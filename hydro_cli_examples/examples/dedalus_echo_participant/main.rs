use hydroflow::util::{
    cli::{ConnectedBidi, ConnectedDemux, ConnectedSink, ConnectedSource},
    deserialize_from_bytes, serialize_to_bytes,
};
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let to_replica_source = ports
        .remove("to_replica")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .into_source();

    let from_replica_port = ports
        .remove("from_replica")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;

    let peers = from_replica_port.keys.clone();
    let from_replica_sink = from_replica_port.into_sink();

    let mut df = datalog!(
        r#"
        .input leader `repeat_iter(peers.clone()) -> map(|p| (p,))`
.async voteToReplica `null::<(u32,)>()` `source_stream(to_replica_source) -> map(|x| deserialize_from_bytes::<(u32,)>(x.unwrap()).unwrap())`
.async voteFromReplica `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(from_replica_sink)` `null::<(u32,)>()`
            
voteFromReplica@addr(v) :~ voteToReplica(v), leader(addr)
        "#
    );

    df.run_async().await;
}
