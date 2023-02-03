use hydroflow::hydroflow_syntax;
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let timer = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
        Duration::from_millis(1000),
    ));

    let mut df = hydroflow_syntax! {
        repeat_iter(0..5) -> buffer(timer) -> for_each(|x| { println!("{x:?}"); });
    };

    df.run_async().await;
}
