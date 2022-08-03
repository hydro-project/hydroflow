use hydroflow::hydroflow_syntax;

pub fn test_surface_syntax_strata_expand() {}

// RUSTFLAGS="$RUSTFLAGS -Z proc-macro-backtrace" cargo test --package hydroflow --test surface_stratum -- test_surface_syntax_strata --exact --nocapture
#[test]
pub fn test_surface_syntax_strata() {
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pairs_send, pairs_recv) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();

    // let mut df = hydroflow_syntax! {
    //     reached_vertices = merge() -> map(|v| (v, ()));
    //     recv_iter(vec![0]) -> [0]reached_vertices;

    //     edges = merge() -> tee();
    //     recv_stream(pairs_recv) -> [0]edges;

    //     my_join_tee = join() -> map(|(_src, ((), dst))| dst) -> map(|x| x) -> map(|x| x) -> tee();
    //     reached_vertices -> [0]my_join_tee;
    //     edges[0] -> [1]my_join_tee;

    //     my_join_tee[0] -> [1]reached_vertices;

    //     diff_out = difference() -> tee();

    //     edges[1] -> flat_map(|(a, b)| [a, b]) -> [0]diff_out;
    //     my_join_tee[1] -> [1]diff_out;

    //     diff_out[0] -> for_each(|x| println!("Not reached: {}", x));
    //     diff_out[1] -> map(|x| (0, x)) -> [1]edges;
    // };
    let mut df = hydroflow_syntax! {
        reached_vertices = merge() -> map(|v| (v, ()));
        recv_iter(vec![0]) -> [0]reached_vertices;

        edges = recv_stream(pairs_recv) -> tee();

        my_join_tee = join() -> map(|(_src, ((), dst))| dst) -> map(|x| x) -> map(|x| x) -> tee();
        reached_vertices -> [0]my_join_tee;
        edges[1] -> [1]my_join_tee;

        my_join_tee[0] -> [1]reached_vertices;

        diff = difference() -> for_each(|x| println!("Not reached: {}", x));

        edges[0] -> flat_map(|(a, b)| [a, b]) -> [0]diff;
        my_join_tee[1] -> [1]diff;
    };

    println!(
        "{}",
        df.serde_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );
    df.run_available();

    println!("A");

    pairs_send.send((0, 1)).unwrap();
    pairs_send.send((2, 4)).unwrap();
    pairs_send.send((3, 4)).unwrap();
    pairs_send.send((1, 2)).unwrap();
    pairs_send.send((0, 3)).unwrap();
    df.run_available();

    // println!("B");

    // pairs_send.send((0, 3)).unwrap();
    // df.run_available();
}
