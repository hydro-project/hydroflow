use std::thread;

use hydroflow::util::collect_ready;
use hydroflow_datalog::datalog;

#[test]
pub fn test_minimal() {
    let (in_send, input) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    in_send.send((1, 2)).unwrap();

    let mut flow = datalog!(
        r#"
        .input input
        .output out

        out(y, x) :- input(x, y).
        "#
    );

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[(2, 1)]);
}

#[test]
pub fn test_join_with_self() {
    let (in_send, input) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    in_send.send((1, 2)).unwrap();
    in_send.send((2, 1)).unwrap();
    in_send.send((1, 3)).unwrap();

    let mut flow = datalog!(
        r#"
        .input input
        .output out

        out(x, y) :- input(x, y), input(y, x).
        "#
    );

    flow.run_tick();

    assert_eq!(
        &*collect_ready::<Vec<_>, _>(&mut out_recv),
        &[(2, 1), (1, 2)]
    );
}

#[test]
pub fn test_multi_use_intermediate() {
    let (in_send, input) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    in_send.send((1, 2)).unwrap();
    in_send.send((2, 1)).unwrap();
    in_send.send((1, 3)).unwrap();

    let mut flow = datalog!(
        r#"
        .input input
        .output out

        in_dup(x, y) :- input(x, y).
        out(x, y) :- in_dup(x, y), in_dup(y, x).
        "#
    );

    flow.run_tick();

    assert_eq!(
        &*collect_ready::<Vec<_>, _>(&mut out_recv),
        &[(2, 1), (1, 2)]
    );
}

#[test]
pub fn test_join_with_other() {
    let (in1_send, in1) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (in2_send, in2) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut flow = datalog!(
        r#"
        .input in1
        .input in2
        .output out

        out(x, y) :- in1(x, y), in2(y, x).
        "#
    );

    in1_send.send((1, 2)).unwrap();
    in1_send.send((1, 3)).unwrap();
    in2_send.send((2, 1)).unwrap();
    in2_send.send((4, 1)).unwrap();

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[(1, 2)]);

    in1_send.send((1, 3)).unwrap();
    in1_send.send((1, 4)).unwrap();
    in2_send.send((3, 1)).unwrap();

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[(1, 3)]);
}

#[test]
pub fn test_multiple_contributors() {
    let (in1_send, in1) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (in2_send, in2) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    in1_send.send((1, 2)).unwrap();
    in2_send.send((3, 1)).unwrap();

    let mut flow = datalog!(
        r#"
        .input in1
        .input in2
        .output out

        out(x, y) :- in1(x, y).
        out(x, y) :- in2(y, x).
        "#
    );

    flow.run_tick();

    assert_eq!(
        &*collect_ready::<Vec<_>, _>(&mut out_recv),
        &[(1, 2), (1, 3)]
    );
}

#[test]
pub fn test_transitive_closure() {
    let (edges_send, edges) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (seed_reachable_send, seed_reachable) = hydroflow::util::unbounded_channel::<(usize,)>();
    let (reachable, mut reachable_recv) = hydroflow::util::unbounded_channel::<(usize,)>();

    seed_reachable_send.send((1,)).unwrap();
    edges_send.send((3, 4)).unwrap();
    edges_send.send((1, 2)).unwrap();
    edges_send.send((2, 5)).unwrap();

    let mut flow = datalog!(
        r#"
        .input edges
        .input seed_reachable
        .output reachable

        reachable(x) :- seed_reachable(x).
        reachable(y) :- reachable(x), edges(x, y).
        "#
    );

    flow.run_tick();

    assert_eq!(
        &*collect_ready::<Vec<_>, _>(&mut reachable_recv),
        &[(1,), (2,), (5,)]
    );
}

#[test]
pub fn test_triple_relation_join() {
    let (in1_send, in1) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (in2_send, in2) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (in3_send, in3) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize, usize, usize)>();

    in1_send.send((1, 2)).unwrap();
    in2_send.send((2, 1)).unwrap();

    in3_send.send((1, 3)).unwrap();
    in3_send.send((1, 4)).unwrap();
    in3_send.send((2, 3)).unwrap();

    let mut flow = datalog!(
        r#"
        .input in1
        .input in2
        .input in3
        .output out

        out(d, c, b, a) :- in1(a, b), in2(b, c), in3(c, d).
        "#
    );

    flow.run_tick();

    assert_eq!(
        &*collect_ready::<Vec<_>, _>(&mut out_recv),
        &[(3, 1, 2, 1), (4, 1, 2, 1)]
    );
}

#[test]
pub fn test_local_constraints() {
    let (in_send, input) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    in_send.send((1, 2)).unwrap();
    in_send.send((1, 1)).unwrap();

    let mut flow = datalog!(
        r#"
        .input input
        .output out

        out(x, x) :- input(x, x).
        "#
    );

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[(1, 1)]);
}

#[test]
pub fn test_boolean_relation_eq() {
    let (in_send, input) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    in_send.send((1, 1)).unwrap();
    in_send.send((1, 2)).unwrap();
    in_send.send((2, 1)).unwrap();

    thread::spawn(|| {
        let mut flow = datalog!(
            r#"
            .input input
            .output out

            out(a, b) :- input(a, b), ( a == b ).
            "#
        );

        flow.run_tick();
    })
    .join()
    .unwrap();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[(1, 1)]);
}

#[test]
pub fn test_boolean_relation_lt() {
    let (in_send, input) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    in_send.send((1, 1)).unwrap();
    in_send.send((1, 2)).unwrap();
    in_send.send((2, 1)).unwrap();

    let mut flow = datalog!(
        r#"
        .input input
        .output out

        out(a, b) :- input(a, b), ( a < b ).
        "#
    );

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[(1, 2)]);
}

#[test]
pub fn test_boolean_relation_le() {
    let (in_send, input) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    in_send.send((1, 1)).unwrap();
    in_send.send((1, 2)).unwrap();
    in_send.send((2, 1)).unwrap();

    let mut flow = datalog!(
        r#"
        .input input
        .output out

        out(a, b) :- input(a, b), ( a <= b ).
        "#
    );

    flow.run_tick();

    assert_eq!(
        &*collect_ready::<Vec<_>, _>(&mut out_recv),
        &[(1, 1), (1, 2)]
    );
}

#[test]
pub fn test_boolean_relation_gt() {
    let (in_send, input) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    in_send.send((1, 1)).unwrap();
    in_send.send((1, 2)).unwrap();
    in_send.send((2, 1)).unwrap();

    let mut flow = datalog!(
        r#"
        .input input
        .output out

        out(a, b) :- input(a, b), ( a > b ).
        "#
    );

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[(2, 1)]);
}

#[test]
pub fn test_boolean_relation_ge() {
    let (in_send, input) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    in_send.send((1, 1)).unwrap();
    in_send.send((1, 2)).unwrap();
    in_send.send((2, 1)).unwrap();

    let mut flow = datalog!(
        r#"
        .input input
        .output out

        out(a, b) :- input(a, b), ( a >= b ).
        "#
    );

    flow.run_tick();

    assert_eq!(
        &*collect_ready::<Vec<_>, _>(&mut out_recv),
        &[(1, 1), (2, 1)]
    );
}

#[test]
pub fn test_join_multiple_and_relation() {
    let (in1_send, in1) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (in2_send, in2) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (in3_send, in3) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize, usize, usize)>();

    in1_send.send((1, 2)).unwrap();

    in2_send.send((2, 3)).unwrap();
    in2_send.send((2, 4)).unwrap();

    in3_send.send((3, 4)).unwrap();
    in3_send.send((4, 5)).unwrap();
    in3_send.send((4, 0)).unwrap();

    let mut flow = datalog!(
        r#"
        .input in1
        .input in2
        .input in3
        .output out

        out(a, b, c, d) :- in1(a, b), in2(b, c), in3(c, d), ( d > a ).
        "#
    );
    flow.run_tick();

    assert_eq!(
        &*collect_ready::<Vec<_>, _>(&mut out_recv),
        &[(1, 2, 3, 4), (1, 2, 4, 5)]
    );
}

#[test]
pub fn test_join_multiple_then_relation() {
    // Same test as test_join_multiple_and_relation, except with a filter on top instead of a
    // filter in the join.
    let (in1_send, in1) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (in2_send, in2) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (in3_send, in3) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize, usize, usize)>();

    in1_send.send((1, 2)).unwrap();

    in2_send.send((2, 3)).unwrap();
    in2_send.send((2, 4)).unwrap();

    in3_send.send((3, 4)).unwrap();
    in3_send.send((4, 5)).unwrap();
    in3_send.send((4, 0)).unwrap();

    let mut flow = datalog!(
        r#"
        .input in1
        .input in2
        .input in3
        .output out

        int(a, b, c, d) :- in1(a, b), in2(b, c), in3(c, d).
        out(a, b, c, d) :- int(a, b, c, d), ( d > a ).
        "#
    );
    flow.run_tick();

    assert_eq!(
        &*collect_ready::<Vec<_>, _>(&mut out_recv),
        &[(1, 2, 3, 4), (1, 2, 4, 5)]
    );
}

#[test]
pub fn test_next_tick() {
    let (ints_1_send, ints_1) = hydroflow::util::unbounded_channel::<(usize,)>();
    let (ints_2_send, ints_2) = hydroflow::util::unbounded_channel::<(usize,)>();
    let (result, mut result_recv) = hydroflow::util::unbounded_channel::<(usize,)>();

    ints_1_send.send((1,)).unwrap();
    ints_1_send.send((2,)).unwrap();
    ints_2_send.send((3,)).unwrap();
    ints_2_send.send((4,)).unwrap();

    let mut flow = datalog!(
        r#"
        .input ints_1
        .input ints_2
        .output result

        result(x) :- ints_1(x).
        result(x) :+ ints_2(x).
        "#
    );

    flow.run_tick();

    assert_eq!(
        &*collect_ready::<Vec<_>, _>(&mut result_recv),
        &[(1,), (2,)]
    );

    flow.run_tick();

    assert_eq!(
        &*collect_ready::<Vec<_>, _>(&mut result_recv),
        &[(3,), (4,)]
    );
}

#[test]
pub fn test_anti_join() {
    let (ints_1_send, ints_1) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (ints_2_send, ints_2) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (ints_3_send, ints_3) = hydroflow::util::unbounded_channel::<(usize,)>();
    let (result, mut result_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut flow = datalog!(
        r#"
        .input ints_1
        .input ints_2
        .input ints_3
        .output result

        result(x, z) :- ints_1(x, y), ints_2(y, z), !ints_3(y)
        "#
    );

    ints_1_send.send((1, 2)).unwrap();
    ints_1_send.send((2, 3)).unwrap();
    ints_2_send.send((2, 3)).unwrap();
    ints_2_send.send((3, 4)).unwrap();
    ints_3_send.send((2,)).unwrap();

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut result_recv), &[(2, 4)]);

    ints_1_send.send((1, 2)).unwrap();
    ints_1_send.send((2, 3)).unwrap();
    ints_2_send.send((2, 3)).unwrap();
    ints_2_send.send((3, 4)).unwrap();
    ints_3_send.send((3,)).unwrap();

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut result_recv), &[(1, 3)]);
}

#[test]
pub fn test_anti_join_next_tick() {
    let (ints_1_send, ints_1) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (ints_2_send, ints_2) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (ints_3_send, ints_3) = hydroflow::util::unbounded_channel::<(usize,)>();
    let (result, mut result_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut flow = datalog!(
        r#"
        .input ints_1
        .input ints_2
        .input ints_3
        .output result

        result(x, z) :+ ints_1(x, y), ints_2(y, z), !ints_3(y)
        "#
    );

    ints_1_send.send((1, 2)).unwrap();
    ints_1_send.send((2, 3)).unwrap();
    ints_2_send.send((2, 3)).unwrap();
    ints_2_send.send((3, 4)).unwrap();
    ints_3_send.send((2,)).unwrap();

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut result_recv), &[]);

    ints_1_send.send((1, 2)).unwrap();
    ints_1_send.send((2, 3)).unwrap();
    ints_2_send.send((2, 3)).unwrap();
    ints_2_send.send((3, 4)).unwrap();
    ints_3_send.send((3,)).unwrap();

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut result_recv), &[(2, 4)]);

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut result_recv), &[(1, 3)]);
}

#[test]
pub fn test_anti_join_next_tick_cycle() {
    let (ints_1_send, ints_1) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (ints_2_send, ints_2) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (ints_3_send, ints_3) = hydroflow::util::unbounded_channel::<(usize,)>();
    let (result, mut result_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut flow = datalog!(
        r#"
        .input ints_1
        .input ints_2
        .input ints_3
        .output result

        result(x, z) :+ ints_1(x, y), ints_2(y, z), !ints_3(y), !result(x, z)
        "#
    );

    ints_1_send.send((1, 2)).unwrap();
    ints_1_send.send((2, 3)).unwrap();
    ints_2_send.send((2, 3)).unwrap();
    ints_2_send.send((3, 4)).unwrap();
    ints_3_send.send((2,)).unwrap();

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut result_recv), &[]);

    ints_1_send.send((1, 2)).unwrap();
    ints_1_send.send((2, 3)).unwrap();
    ints_2_send.send((2, 3)).unwrap();
    ints_2_send.send((3, 4)).unwrap();

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut result_recv), &[(2, 4)]);

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut result_recv), &[(1, 3)]);
}

#[test]
fn test_max() {
    let (ints_send, ints) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (result, mut result_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut flow = datalog!(
        r#"
        .input ints
        .output result

        result(max(a), b) :- ints(a, b)
        "#
    );

    ints_send.send((1, 2)).unwrap();
    ints_send.send((2, 2)).unwrap();
    ints_send.send((3, 2)).unwrap();

    ints_send.send((3, 3)).unwrap();
    ints_send.send((4, 3)).unwrap();
    ints_send.send((5, 3)).unwrap();

    flow.run_tick();

    let mut res = collect_ready::<Vec<_>, _>(&mut result_recv);
    res.sort_by_key(|v| v.0);
    assert_eq!(&res, &[(3, 2), (5, 3)]);
}

#[test]
fn test_max_all() {
    let (ints_send, ints) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (result, mut result_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut flow = datalog!(
        r#"
        .input ints
        .output result

        result(max(a), max(b)) :- ints(a, b)
        "#
    );

    ints_send.send((1, 3)).unwrap();
    ints_send.send((2, 2)).unwrap();
    ints_send.send((3, 1)).unwrap();

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut result_recv), &[(3, 3)]);
}

#[test]
fn test_max_next_tick() {
    let (ints_send, ints) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (result, mut result_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut flow = datalog!(
        r#"
        .input ints
        .output result

        result(max(a), max(b)) :+ ints(a, b)
        "#
    );

    ints_send.send((1, 3)).unwrap();
    ints_send.send((2, 2)).unwrap();
    ints_send.send((3, 1)).unwrap();

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut result_recv), &[]);

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut result_recv), &[(3, 3)]);
}

#[test]
fn test_send_to_node() {
    let (ints_send_1, ints_1) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (_ints_send_2, ints_2) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let (result_1, mut result_recv_1) = hydroflow::util::unbounded_channel::<(usize,)>();
    let (result_2, mut result_recv_2) = hydroflow::util::unbounded_channel::<(usize,)>();

    let (_async_send_result_1, async_receive_result_1) =
        hydroflow::util::unbounded_channel::<(usize,)>();
    let (async_send_result_2, async_receive_result_2) =
        hydroflow::util::unbounded_channel::<(usize,)>();

    let mut flow_1 = {
        let ints = ints_1;
        let async_receive_result = async_receive_result_1;
        let result = result_1;

        let send_to_node = move |node: usize, data: (usize,)| -> Result<(), ()> {
            assert!(node == 2);
            async_send_result_2.send(data).unwrap();
            Ok(())
        };

        datalog!(
            r#"
            .input ints
            .output result

            result@b(a) :~ ints(a, b)
            "#
        )
    };

    let mut flow_2 = {
        let ints = ints_2;
        let async_receive_result = async_receive_result_2;
        let result = result_2;

        let send_to_node = |_: usize, _: (usize,)| -> Result<(), ()> {
            panic!("Should not be called");
        };

        datalog!(
            r#"
            .input ints
            .output result

            result@b(a) :~ ints(a, b)
            "#
        )
    };

    ints_send_1.send((5, 2)).unwrap();

    flow_1.run_tick();
    flow_2.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut result_recv_1), &[]);
    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut result_recv_2), &[(5,)]);
}
