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
            -> filter(|&x| x == "hello") -> for_each(|x| println!("{}", x));
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

#[test]
pub fn test_example1() {
    let mut flow = hydroflow_syntax! {
        recv_iter(0..10) -> for_each(|n| println!("Hello {}", n));
    };

    flow.run_available();
}

#[test]
pub fn test_example2() {
    let mut flow = hydroflow_syntax! {
        recv_iter(0..10)
            -> map(|n| n * n)
            -> filter(|&n| n > 10)
            -> map(|n| (n..=n+1))
            -> flat_map(|n| n)
            -> for_each(|n| println!("Howdy {}", n));
    };

    flow.run_available();
}

#[test]
pub fn test_example2_1() {
    let mut flow = hydroflow_syntax! {
        recv_iter(0..10)
        -> filter_map(|n| {
            let n2 = n * n;
            if n2 > 10 {
                Some(n2)
            }
            else {
                None
            }
        })
        -> flat_map(|n| (n..=n+1))
        -> for_each(|n| println!("G'day {}", n))
    };

    flow.run_available();
}

#[test]
pub fn test_example3() {
    // Create our channel input
    let (input_example, example_recv) = tokio::sync::mpsc::unbounded_channel::<usize>();

    let mut hydroflow = hydroflow_syntax! {
        recv_stream(example_recv)
        -> filter_map(|n: usize| {
            let n2 = n * n;
            if n2 > 10 {
                Some(n2)
            }
            else {
                None
            }
        })
        -> flat_map(|n| (n..=n+1))
        -> for_each(|n| println!("G'day {}", n))
    };

    println!("A");
    input_example.send(0).unwrap();
    input_example.send(1).unwrap();
    input_example.send(2).unwrap();
    input_example.send(3).unwrap();
    input_example.send(4).unwrap();
    input_example.send(5).unwrap();

    hydroflow.run_available();

    println!("B");
    input_example.send(6).unwrap();
    input_example.send(7).unwrap();
    input_example.send(8).unwrap();
    input_example.send(9).unwrap();

    hydroflow.run_available();
}

#[test]
pub fn test_example4() {
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pairs_send, pairs_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    let mut flow = hydroflow_syntax! {
        // the origin vertex: node 0
        origin = recv_iter(vec![0]);
        reached_vertices = merge() -> map(|v| (v, ()));

        origin -> [0]reached_vertices;

        my_join_tee = join() -> map(|(_src, ((), dst))| dst) -> tee();
        reached_vertices -> [0]my_join_tee;
        recv_stream(pairs_recv) -> [1]my_join_tee;

        my_join_tee[0] -> [1]reached_vertices;
        my_join_tee[1] -> for_each(|x| println!("Reached: {}", x));
    };

    println!(
        "{}",
        flow.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    pairs_send.send((0, 1)).unwrap();
    pairs_send.send((2, 4)).unwrap();
    pairs_send.send((3, 4)).unwrap();
    pairs_send.send((1, 2)).unwrap();
    pairs_send.send((0, 3)).unwrap();
    pairs_send.send((0, 3)).unwrap();
    flow.run_available();
    // Reached: 1
    // Reached: 2
    // Reached: 4
    // Reached: 3
    // Reached: 4
}

async fn echo_test() {
    let (stimulus_in, stimulus_out) = tokio::sync::mpsc::unbounded_channel::<usize>();
    let (response_in, response_out) = tokio::sync::mpsc::unbounded_channel::<usize>();

    let local = tokio::task::LocalSet::new();

    // echo server
    local.spawn_local(async move {
        let mut server = hydroflow_syntax! {
            outs = tee();
            recv_stream(stimulus_out) -> outs;
            outs[0] -> for_each(|x| println!("Server got {}", x));
            outs[1] -> for_each(|x| { response_in.send(x).unwrap(); });
        };
        server.run_async().await.unwrap();
    });

    // client
    local.spawn_local(async move {
        let mut client = hydroflow_syntax! {
            recv_iter(1..4) -> for_each(|x| { stimulus_in.send(x).unwrap();} );
            recv_stream(response_out) -> for_each(|x| println!("Client got echo {}", x));
        };
        client.run_async().await.unwrap();
    });

    local.await;
}
