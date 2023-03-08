use hydroflow::util::serialize_to_bytes;
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let foo_send = ports.remove("foo").unwrap().1.unwrap();

    let mut df = datalog!(
        r#"
        .input repeated `repeat_iter(["Hello".to_string(), "world".to_string()]) -> map(|x| (x,))`
        .output foo `map(|(v,)| serialize_to_bytes(v)) -> dest_sink(foo_send)`

        foo(x) :- repeated(x)
    "#
    );

    df.run_async().await;
}
