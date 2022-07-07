use hydroflow::{hydroflow_parser, hydroflow_syntax};

#[test]
pub fn test_surface_syntax() {
    hydroflow_syntax! {
        reached_vertices = (merge() -> map(|v| (v, ())));
        (seed([0]) -> [0]reached_vertices);

        my_join = (join() -> map(|(_src, ((), dst))| dst) -> dedup() -> tee());
        (reached_vertices -> [0]my_join);
        (input(/*(v, v) edges*/) -> [1]my_join);

        (my_join[0] -> [1]reached_vertices);
        (my_join[1] -> for_each(|x| println!("Reached: {}", x)));
    };

    // // A dataflow that represents graph reachability.
    // let mut df = Hydroflow::new();

    // let (reachable_out, origins_in) =
    //     df.make_edge::<_, VecHandoff<usize>>("reachable -> origins");
    // let (did_reach_out, possible_reach_in) =
    //     df.make_edge::<_, VecHandoff<usize>>("did_reach -> possible_reach");
    // let (output_out, sink_in) = df.make_edge::<_, VecHandoff<usize>>("output -> sink");

    // df.add_subgraph_source(
    //     "initially reachable source",
    //     reachable_out,
    //     move |_ctx, send| {
    //         send.give(Some(1));
    //     },
    // );

    // let seen_handle = df.add_state::<RefCell<HashSet<usize>>>(Default::default());

    // df.add_subgraph(
    //     "main",
    //     tl!(origins_in, possible_reach_in),
    //     tl!(did_reach_out, output_out),
    //     move |context, tl!(origins, did_reach_recv), tl!(did_reach_send, output)| {
    //         let origins = origins.take_inner().into_iter();
    //         let possible_reach = did_reach_recv
    //             .take_inner()
    //             .into_iter()
    //             .filter_map(|v| edges.get(&v))
    //             .flatten()
    //             .copied();

    //         let mut seen_state = context.state_ref(seen_handle).borrow_mut();
    //         let pull = origins
    //             .chain(possible_reach)
    //             .filter(|v| seen_state.insert(*v));

    //         let pivot = pull
    //             .pull_to_push()
    //             .tee(ForEach::new(|v| {
    //                 did_reach_send.give(Some(v));
    //             }))
    //             .for_each(|v| {
    //                 output.give(Some(v));
    //             });

    //         pivot.run();
    //     },
    // );

    // let reachable_verts = Rc::new(RefCell::new(HashSet::new()));
    // let reachable_inner = reachable_verts.clone();
    // df.add_subgraph_sink("output sink", sink_in, move |_ctx, recv| {
    //     (*reachable_inner).borrow_mut().extend(recv.take_inner());
    // });

    // df.run_available();
}

// #[test]
// pub fn test_parser_basic() {
//     println!("DONE");

//     hydroflow_parser! {
//         reached_vertices = (merge() -> map(|v| (v, ())));
//         (seed([0]) -> [0]reached_vertices);

//         my_join = (join() -> map(|(_src, ((), dst))| dst) -> dedup() -> tee());
//         (reached_vertices -> [0]my_join);
//         (input(/*(v, v) edges*/) -> [1]my_join);

//         (my_join[0] -> [1]reached_vertices);
//         (my_join[1] -> for_each(|x| println!("Reached: {}", x)));
//     }

//     hydroflow_parser! {
//         shuffle = (merge() -> tee());
//         (shuffle[0] -> [0]shuffle);
//         (shuffle[1] -> [1]shuffle);
//         (shuffle[2] -> [2]shuffle);
//         (shuffle[3] -> [3]shuffle);
//     }

//     hydroflow_parser! {
//         x = (map(a) -> map(b));
//         (x -> x);
//     }

//     hydroflow_parser! {
//         a = map(a); // 0
//         b = (merge() -> tee()); // 1
//         c = merge(); // 2
//         d = tee(); // 3
//         e = (merge() -> tee()); // 4
//         f = map(f); // 5
//         g = merge(); // 6
//         h = tee(); // 7

//         (a -> b);

//         (b -> e);
//         (b -> g);

//         (c -> b);

//         (d -> a);
//         (d -> b);
//         (d -> e);

//         (e -> c);
//         (e -> h);

//         (f -> c);

//         (g -> e);

//         (h -> d);
//         (h -> f);
//         (h -> g);
//     }
// }
