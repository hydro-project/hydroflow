use hydroflow::hydroflow_syntax;

pub fn main() {
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pairs_send, pairs_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut flow = hydroflow_syntax! {
        origin = source_iter(vec![0]);
        stream_of_edges = source_stream(pairs_recv) -> tee();
        reached_vertices = union() -> unique() -> tee();
        origin -> reached_vertices;

        // the join for reachable vertices
        my_join = join() -> flat_map(|(src, ((), dst))| [src, dst]);
        reached_vertices -> map(|v| (v, ())) -> [0]my_join;
        stream_of_edges[1] -> [1]my_join;

        // the loop
        my_join -> reached_vertices;

        // the difference all_vertices - reached_vertices
        all_vertices = stream_of_edges[0]
          -> flat_map(|(src, dst)| [src, dst]) -> tee();
        unreached_vertices = difference();
        all_vertices -> [pos]unreached_vertices;
        reached_vertices -> [neg]unreached_vertices;

        // the output
        all_vertices -> for_each(|v| println!("Received vertex: {}", v));
        unreached_vertices -> for_each(|v| println!("unreached_vertices vertex: {}", v));
    };

    println!(
        "{}",
        flow.meta_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );

    pairs_send.send((5, 10)).unwrap();
    pairs_send.send((0, 3)).unwrap();
    pairs_send.send((3, 6)).unwrap();
    pairs_send.send((6, 5)).unwrap();
    pairs_send.send((11, 12)).unwrap();
    flow.run_available();
}
