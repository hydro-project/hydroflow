use std::collections::HashMap;

use hydroflow::{hydroflow_syntax, util::connection::ConnectionPipe};

#[tokio::main]
async fn main() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();

    let mut connection_pipes =
        serde_json::from_str::<HashMap<String, ConnectionPipe>>(trimmed).unwrap();

    // bind to sockets
    let server = connection_pipes.remove("bar").unwrap().bind().await;

    println!("ready");

    // listen for incoming connections
    let (_, bar_recv) = server.accept_lines().await;

    let mut start_buf = String::new();
    std::io::stdin().read_line(&mut start_buf).unwrap();
    if start_buf != "start\n" {
        panic!("expected start");
    }

    // connect to sockets

    // start program
    let mut df = hydroflow_syntax! {
        bar = source_stream(bar_recv)
            -> map(|x| x.unwrap())
            -> tee();

        bar[0] -> for_each(|s| println!("echo {}", s));
    };

    df.run_async().await;
}
