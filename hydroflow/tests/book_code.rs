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

#[test]
pub fn test_filter() {
    let mut flow = hydroflow_syntax! {
        recv_iter(vec!["hello", "world"])
            -> filter(|x| x == &"hello") -> for_each(|x| println!("{}", x));
    };
    flow.run_available();
}

#[test]
pub fn test_flat_map() {
    let mut flow = hydroflow_syntax! {
        recv_iter(vec!["hello", "world"])
            -> flat_map(|x| x.chars()) -> for_each(|x| println!("{}", x));
    };
    flow.run_available();
}

#[test]
pub fn test_filter_map() {
    let mut flow = hydroflow_syntax! {
        recv_iter(vec!["1", "hello", "world", "2"])
            -> filter_map(|s| s.parse().ok()) -> for_each(|x: usize| println!("{}", x));
    };
    flow.run_available();
}

#[test]
pub fn test_merge() {
    let mut flow = hydroflow_syntax! {
        my_merge = merge();
        recv_iter(vec!["hello", "world"]) -> [0]my_merge;
        recv_iter(vec!["stay", "gold"]) -> [1]my_merge;
        recv_iter(vec!["don\'t", "give", "up"]) -> [2]my_merge;
        my_merge -> map(|x| x.to_uppercase()) -> for_each(|x| println!("{}", x));
    };
    flow.run_available();
}

#[test]
pub fn test_join() {
    let mut flow = hydroflow_syntax! {
        my_join = join();
        recv_iter(vec![("hello", "world"), ("stay", "gold")]) -> [0]my_join;
        recv_iter(vec![("hello", "cleveland")]) -> [1]my_join;
        my_join -> for_each(|(k, (v1, v2))| println!("({}, ({}, {})", k, v1, v2));
    };
    flow.run_available();
}

#[test]
pub fn test_tee() {
    let mut flow = hydroflow_syntax! {
        my_tee = recv_iter(vec!["hello", "world"]) -> tee();
        my_tee[0] -> map(|x: &str| x.to_uppercase())
            -> for_each(|x| println!("{}", x));
        my_tee[1] -> map(|x: &str| x.to_lowercase())
            -> for_each(|x| println!("{}", x));
        my_tee[2] -> for_each(|x: &str| println!("{}", x));
    };
    flow.run_available();
}
