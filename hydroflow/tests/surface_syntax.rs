use hydroflow::hydroflow_parser;

#[test]
pub fn test_parser_basic() {
    // hydroflow_parser! {
    //     edges_input = (input() ->);

    //     init_vertex = (seed([0]) ->);
    //     // loop_vertices = (->);
    //     out_vertices = (-> for_each(|x| println!("Reached: {}", x)));

    //     reached_vertices = (merge[init_vertex, loop_vertices] -> map(|v| (v, ())));

    //     (join[reached_vertices, edges_input] -> map(|(_src, ((), dst))| dst) -> dedup() -> tee[loop_vertices, out_vertices]);

    //     // x = (a -> b() -> c() -> (a -> b -> c) -> p);
    //     // b = (a -> b() -> c() -> (a -> b -> c) -> p);
    //     // x = (a -> b() -> c() -> (a -> b -> c) -> p);
    //     // x = (a -> b() -> c() -> (a -> b -> c) -> p);
    // }

    hydroflow_parser! {
        edges_input = input();
        init_vertex = seed([0]);
        out_vertices = for_each(|x| println!("Reached: {}", x));

        // reached_vertices = (merge[init_vertex, loop_vertices] -> map(|v| (v, ())));
        reached_vertices = (merge() -> map(|v| (v, ())));
        (init_vertex -> [0]reached_vertices);


        map_dedup = (map(|(_src, ((), dst))| dst) -> dedup());
        my_join = (join() -> map_dedup);
        (reached_vertices -> [0]my_join);
        (edges_input -> [1]my_join);

        my_join_tee = (my_join -> tee());
        (my_join_tee[0] -> [1]reached_vertices);
        (my_join_tee[1] -> out_vertices);

        shuffle = (merge() -> tee());
        (shuffle[0] -> [0]shuffle);
        (shuffle[1] -> [1]shuffle);
        (a -> [2]shuffle);
        (b -> [3]shuffle);
        (c -> [4]shuffle);
        // (a -> [0]shuffle[0] -> b);
        // (c -> [0]shuffle[0] -> d);
    }
}
