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

    let mut bind_types = serde_json::from_str::<HashMap<String, BindType>>(trimmed).unwrap();

    // bind to sockets
    let mut bind_results: HashMap<String, ConnectionPipe> = HashMap::new();
    let bar_bind = bind_types.remove("bar").unwrap().bind().await;
    bind_results.insert("bar".to_string(), bar_bind.connection_pipe());

    let bind_serialized = serde_json::to_string(&bind_results).unwrap();
    println!("ready: {bind_serialized}");

    // listen for incoming connections
    let (_, bar_recv) = bar_bind.accept_lines().await;

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

    // start program
    let mut df = hydroflow_syntax! {
        bar = source_stream(bar_recv)
            -> map(|x| x.unwrap())
            -> tee();

        bar[0] -> for_each(|s| println!("echo {}", s));
    };

    df.run_async().await;
}
