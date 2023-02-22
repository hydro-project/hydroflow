use std::collections::HashMap;

use hydroflow::{
    hydroflow_syntax,
    util::connection::ConnectionPipe,
};

#[tokio::main]
async fn main() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();

    let connection_pipes =
        serde_json::from_str::<HashMap<String, ConnectionPipe>>(trimmed).unwrap();

    // bind to sockets

    println!("ready");

    // listen for incoming connections

    let mut start_buf = String::new();
    std::io::stdin().read_line(&mut start_buf).unwrap();
    if start_buf != "start\n" {
        panic!("expected start");
    }

    // connect to sockets
    let (foo_send, _) = connection_pipes.get("foo").unwrap().connect().await;

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
