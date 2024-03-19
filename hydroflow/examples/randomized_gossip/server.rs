use std::collections::HashSet;
use chrono::Utc;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::util::{UdpSink, UdpStream};
use hydroflow_macro::hydroflow_syntax;
use crate::Opts;
use crate::protocol::ChatMessage;

enum Operation {
    InfectForMessage { msg: ChatMessage },
    RemoveForMessage { msg: ChatMessage },
}

pub(crate) async fn run_server(opts: Opts) {
    println!("Server live!");

    let mut hf = hydroflow_syntax! {

        infected_for_messages = fold::<'static>(
            HashSet::<ChatMessage>::new,
            |current_messages, new_message: Operation | {
                match new_message {
                    Operation::InfectForMessage{ msg } =>  current_messages.insert(msg),
                    Operation::RemoveForMessage{ msg } =>  current_messages.remove(&msg),
            }}) -> for_each (|x| println!("{:?}", x));

        source_iter([Operation::InfectForMessage { msg: ChatMessage { message: "Foo".to_owned(), nickname: "Rohit".to_owned(), ts: Utc::now() } },Operation::InfectForMessage { msg: ChatMessage { message: "Foo".to_owned(), nickname: "Rohit".to_owned(), ts: Utc::now() } }])
            -> infected_for_messages;
    };

    if let Some(graph) = opts.graph {
        let serde_graph = hf
            .meta_graph()
            .expect("No graph found, maybe failed to parse.");
        serde_graph.open_graph(graph, opts.write_config).unwrap();
    }

    hf.run_async().await.unwrap();
}