use hydroflow::hydroflow_parser;

#[test]
pub fn test_parser_basic() {
    println!("DONE");

    hydroflow_parser! {
        reached_vertices = (merge() -> map(|v| (v, ())));
        (recv_iter([0]) -> [0]reached_vertices);

        my_join = (join() -> map(|(_src, ((), dst))| dst) -> tee());
        (reached_vertices -> [0]my_join);
        (recv_stream(/*(v, v) edges*/) -> [1]my_join);

        (my_join[0] -> [1]reached_vertices);
        (my_join[1] -> for_each(|x| println!("Reached: {}", x)));
    }

    hydroflow_parser! {
        shuffle = (merge() -> tee());
        (shuffle[0] -> [0]shuffle);
        (shuffle[1] -> [1]shuffle);
        (shuffle[2] -> [2]shuffle);
        (shuffle[3] -> [3]shuffle);
    }

    hydroflow_parser! {
        x = (map(a) -> map(b));
        (x -> x);
    }

    hydroflow_parser! {
        a = map(a); // 0
        b = (merge() -> tee()); // 1
        c = merge(); // 2
        d = tee(); // 3
        e = (merge() -> tee()); // 4
        f = map(f); // 5
        g = merge(); // 6
        h = tee(); // 7

        (a[0] -> [0]b);

        (b[0] -> [0]e);
        (b[1] -> [0]g);

        (c[0] -> [1]b);

        (d[0] -> [0]a);
        (d[1] -> [2]b);
        (d[2] -> [1]e);

        (e[0] -> [0]c);
        (e[1] -> [0]h);

        (f[0] -> [1]c);

        (g[0] -> [2]e);

        (h[0] -> [0]d);
        (h[1] -> [0]f);
        (h[2] -> [1]g);
    }
}
