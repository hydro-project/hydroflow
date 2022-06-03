use hydroflow::hydroflow_parser;

#[test]
pub fn test_parser_basic() {
    hydroflow_parser! {
        edges_input = (input() ->);

        init_vertex = (seed([0]) ->);
        // loop_vertices = (->);
        out_vertices = (-> for_each(|x| println!("Reached: {}", x)));

        reached_vertices = (merge[init_vertex, loop_vertices] -> map(|v| (v, ())));

        (join[reached_vertices, edges_input] -> map(|(_src, ((), dst))| dst) -> dedup() -> tee[loop_vertices, out_vertices]);


        // x = (a -> b() -> c() -> (a -> b -> c) -> p);
        // b = (a -> b() -> c() -> (a -> b -> c) -> p);
        // x = (a -> b() -> c() -> (a -> b -> c) -> p);
        // x = (a -> b() -> c() -> (a -> b -> c) -> p);
    }
}
