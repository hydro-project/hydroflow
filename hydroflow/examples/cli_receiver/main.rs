use hydroflow::util::deserialize_from_bytes;
use hydroflow_datalog::datalog;

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let bar_recv = ports.remove("bar").unwrap().0.unwrap();

    let mut df = datalog!(
        r#"
        .async repeated `for_each(|_: String| ())` `source_stream(bar_recv) -> map(|x| deserialize_from_bytes::<(String,)>(x.unwrap()))`
        .output stdout `for_each(|tup| println!("echo {:?}", tup))`

        stdout(x) :- repeated(x)
    "#
    );

    df.run_async().await;
}
