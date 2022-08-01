use hydroflow::hydroflow_syntax;

// RUSTFLAGS="$RUSTFLAGS -Z proc-macro-backtrace" cargo test --package hydroflow --test surface_stratum -- test_surface_syntax_strata --exact --nocapture
#[test]
pub fn test_surface_syntax_strata() {
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pairs_send, pairs_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    // let mut df = hydroflow_syntax! {
    //     reached_vertices = merge() -> map(|v| (v, ()));
    //     recv_iter(vec![0]) -> [0]reached_vertices;

    //     my_join_tee = join() -> map(|(_src, ((), dst))| dst) -> map(|x| x) -> next_stratum() -> map(|x| x) -> tee();
    //     reached_vertices -> [0]my_join_tee;
    //     recv_stream(pairs_recv) -> [1]my_join_tee;

    //     my_join_tee[0] -> [1]reached_vertices;
    //     my_join_tee[1] -> next_stratum() -> for_each(|x| println!("Reached: {}", x));
    // };
    let mut df = hydroflow_syntax! {
        reached_vertices = merge() -> map(|v| (v, ()));
        recv_iter(vec![0]) -> [0]reached_vertices;

        my_join_tee = join() -> map(|(_src, ((), dst))| dst) -> map(|x| x) -> map(|x| x) -> tee();
        reached_vertices -> [0]my_join_tee;
        recv_stream(pairs_recv) -> [1]my_join_tee;

        my_join_tee[0] -> [1]reached_vertices;
        my_join_tee[1] -> for_each(|x| println!("Reached: {}", x));
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    df.run_available();

    pairs_send.send((0, 1)).unwrap();
    df.run_available();

    pairs_send.send((2, 4)).unwrap();
    pairs_send.send((3, 4)).unwrap();
    df.run_available();

    pairs_send.send((1, 2)).unwrap();
    df.run_available();

    pairs_send.send((0, 3)).unwrap();
    df.run_available();

    pairs_send.send((0, 3)).unwrap();
    df.run_available();

    // Reached: 1
    // Reached: 2
    // Reached: 4
    // Reached: 3
    // Reached: 4
}
