use hydroflow::{
    tokio_stream::wrappers::IntervalStream,
    util::{
        cli::{ConnectedBidi, ConnectedDemux, ConnectedSink},
        serialize_to_bytes,
    },
};
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let mut broadcast_port = ports
        .remove("broadcast")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;

    let peers: Vec<u32> = serde_json::from_str(&std::env::args().nth(1).unwrap()).unwrap();
    let broadcast_sink = broadcast_port.take_sink();

    let periodic = IntervalStream::new(tokio::time::interval(std::time::Duration::from_secs(1)));

    let mut df = datalog!(
        r#"
        .input repeated `repeat_iter([("Hello".to_string(),), ("world".to_string(),)])`
        .input periodic `source_stream(periodic) -> map(|_| () )`
        .input peers `repeat_iter(peers.clone()) -> map(|p| (p,))`
        .async broadcast `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(broadcast_sink)` `null::<(String,)>()`

        broadcast@n(x) :~ repeated(x), periodic(), peers(n)
    "#
    );

    df.run_async().await;
}
