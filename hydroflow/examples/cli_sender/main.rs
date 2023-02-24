use std::collections::HashMap;

use hydroflow::{
    hydroflow_syntax,
    util::connection::{BindType, ConnectionPipe},
};

#[tokio::main]
async fn main() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();

    #[allow(unused)]
    let mut bind_types = serde_json::from_str::<HashMap<String, BindType>>(trimmed).unwrap();

    // bind to sockets
    #[allow(unused)]
    let mut bind_results: HashMap<String, ConnectionPipe> = HashMap::new();

    let bind_serialized = serde_json::to_string(&bind_results).unwrap();
    println!("ready: {bind_serialized}");

    // listen for incoming connections

    // receive outgoing connection config
    let mut start_buf = String::new();
    std::io::stdin().read_line(&mut start_buf).unwrap();
    let mut connection_pipes = if start_buf.starts_with("start: ") {
        serde_json::from_str::<HashMap<String, ConnectionPipe>>(
            start_buf.trim_start_matches("start: ").trim(),
        )
        .unwrap()
    } else {
        panic!("expected start");
    };

    // connect to sockets
    let (foo_send, _) = connection_pipes.remove("foo").unwrap().connect().await;

    // start program
    let mut df = hydroflow_syntax! {
        foo = merge() -> dest_sink(foo_send);

        repeat_iter([
            "Hello",
            "World",
        ]) -> foo;
    };

    println!("hello from the sender!");

    df.run_async().await;
}
