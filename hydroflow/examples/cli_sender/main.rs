use hydroflow::util::serialize_to_bytes;
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let foo_send = ports.remove("foo").unwrap().1.unwrap();

    let mut df = datalog!(
        r#"
        .input repeated `repeat_iter([("Hello".to_string(),), ("world".to_string(),)])`
        .input peers `repeat_iter([(0,)])`
        .async repeated `map(|(node_id, v)| serialize_to_bytes(v)) -> dest_sink(foo_send)` `null()`

        repeated@n(x) :~ repeated(x), peers(n)
    "#
    );

    df.run_async().await;
}
