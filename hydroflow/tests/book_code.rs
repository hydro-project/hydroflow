use hydroflow::hydroflow_syntax;
#[test]
pub fn test_hello_world() {
    let mut flow = hydroflow_syntax! {
        recv_iter(vec!["hello", "world"])
            -> map(|x| x.to_uppercase()) -> for_each(|x| println!("{}", x));
    };
    println!(
        "{}",
        flow.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    flow.run_available();
}

#[test]
pub fn test_hello_world_twice() {
    let mut flow = hydroflow_syntax! {
        source = recv_iter(vec!["Hello", "world"]) -> tee();
        print = merge() -> for_each(|x| println!("{}", x));
        source[0] -> map(|x: &str| x.to_uppercase()) -> [0]print;
        source[1] -> map(|x: &str| x.to_lowercase()) -> [1]print;
    };
    println!(
        "{}",
        flow.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    flow.run_available();
}

#[test]
pub fn test_hello_world_stream() {
    let (input_send, input_recv) = tokio::sync::mpsc::unbounded_channel::<&str>();
    let mut flow = hydroflow_syntax! {
        recv_stream(input_recv) -> map(|x| x.to_uppercase()) -> for_each(|x| println!("{}", x));
    };
    input_send.send("Hello").unwrap();
    input_send.send("World").unwrap();
    println!(
        "{}",
        flow.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    flow.run_available();
}
