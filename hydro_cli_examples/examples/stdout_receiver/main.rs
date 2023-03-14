use hydroflow::{
    hydroflow_syntax,
    util::cli::{Connected, ConnectedBidi},
};

#[tokio::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
    let echo_recv = ports
        .remove("echo")
        .unwrap()
        .connect::<ConnectedBidi>()
        .await
        .take_source();

    let mut df = hydroflow_syntax! {
        source_stream(echo_recv) -> map(|x| String::from_utf8(x.unwrap().to_vec()).unwrap()) -> for_each(|x| println!("echo {:?}", x));
    };

    df.run_async().await;
}
