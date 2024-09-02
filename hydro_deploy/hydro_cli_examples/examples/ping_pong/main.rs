use hydroflow::hydroflow_syntax;
use hydroflow::util::cli::{ConnectedDirect, ConnectedSource};


#[hydroflow::main]
async fn main() {
    let ports = hydroflow::util::cli::init::<()>().await;
    let echo_recv = ports
        .port("echo_source")
        .connect::<ConnectedDirect>()
        .await
        .into_source();

    let echo_send = ports
        .port("echo_sink")
        .connect::<ConnectedDirect>()
        .await
        .into_sink();

    let df = hydroflow_syntax! {
        source_stream(echo_recv) ->
            map(|x| String::from_utf8(x.unwrap().to_vec()).unwrap()) ->
            for_each(|x| println!("echo {:?}", x)) -> sink;

            sink = dest_sink(echo_send);
        source_iter(vec!["hello".to_string()]) ->
            sink;
    };

    hydroflow::util::cli::launch_flow(df).await;
}
