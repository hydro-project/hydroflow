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
