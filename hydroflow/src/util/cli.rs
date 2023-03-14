use std::collections::HashMap;

use hydroflow_cli_integration::{BoundConnection, ServerConfig, ServerPort};

pub use hydroflow_cli_integration::*;

async fn accept_incoming_connections(
    binds: HashMap<String, BoundConnection>,
) -> HashMap<String, ServerPort> {
    let mut bind_results = HashMap::new();
    for (name, bind) in binds {
        bind_results.insert(name, ServerPort::Bind(bind));
    }
    bind_results
}

pub async fn init() -> HashMap<String, ServerPort> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();

    let bind_config = serde_json::from_str::<HashMap<String, ServerConfig>>(trimmed).unwrap();

    // bind to sockets
    let mut bind_results: HashMap<String, ServerPort> = HashMap::new();
    let mut binds = HashMap::new();
    for (name, config) in bind_config {
        let bound = config.bind().await;
        bind_results.insert(name.clone(), bound.connection_defn());
        binds.insert(name.clone(), bound);
    }

    let bind_connected_future =
        tokio::task::spawn(async move { accept_incoming_connections(binds).await });

    let bind_serialized = serde_json::to_string(&bind_results).unwrap();
    println!("ready: {bind_serialized}");

    let mut start_buf = String::new();
    std::io::stdin().read_line(&mut start_buf).unwrap();
    let connection_defns = if start_buf.starts_with("start: ") {
        serde_json::from_str::<HashMap<String, ServerPort>>(
            start_buf.trim_start_matches("start: ").trim(),
        )
        .unwrap()
    } else {
        panic!("expected start");
    };

    let mut all_connected = HashMap::new();
    for (name, defn) in connection_defns {
        all_connected.insert(name, defn);
    }

    let bind_connected = bind_connected_future.await.unwrap();
    for (name, defn) in bind_connected {
        all_connected.insert(name, defn);
    }

    all_connected
}
