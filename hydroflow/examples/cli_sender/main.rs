use hydroflow::util::{
    cli::{Connected, ConnectedBidi, ConnectedDemux},
    serialize_to_bytes,
};
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let mut foo_port = ports
        .remove("foo")
        .unwrap()
        .connect::<ConnectedDemux<ConnectedBidi>>()
        .await;

    let peers = foo_port.keys.clone();
    let foo_send = foo_port.take_sink();

    let mut df = datalog!(
        r#"
        .input repeated `repeat_iter([("Hello".to_string(),), ("world".to_string(),)])`
        .input peers `repeat_iter(peers.clone()) -> map(|p| (p,))`
        .async broadcast `map(|(node_id, v)| (node_id, serialize_to_bytes(v))) -> dest_sink(foo_send)` `null::<(String,)>()`

        broadcast@n(x) :~ repeated(x), peers(n)
    "#
    );

    df.run_async().await;
}
