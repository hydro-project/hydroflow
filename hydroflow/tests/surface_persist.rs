use multiplatform_test::multiplatform_test;

use hydroflow::util::collect_ready;
use hydroflow::{assert_graphvis_snapshots, hydroflow_syntax};

#[multiplatform_test]
pub fn test_persist_basic() {
    let (result_send, mut result_recv) = hydroflow::util::unbounded_channel::<u32>();

    let mut hf = hydroflow_syntax! {
        repeat_iter([1])
            -> persist()
            -> fold(0, |a, b| (a + b))
            -> for_each(|x| result_send.send(x).unwrap());
    };
    assert_graphvis_snapshots!(hf);

    for tick in 0..10 {
        assert_eq!(tick, hf.current_tick());
        hf.run_tick();
    }
    assert_eq!(
        &[1, 3, 6, 10, 15, 21, 28, 36, 45, 55],
        &*collect_ready::<Vec<_>, _>(&mut result_recv)
    );
}

#[multiplatform_test]
pub fn test_persist_pull() {
    let (result_send, mut result_recv) = hydroflow::util::unbounded_channel::<u32>();

    let mut hf = hydroflow_syntax! {
        // Structured to ensure `persist()` is pull-based.
        repeat_iter([1]) -> m0;
        null() -> m0;
        m0 = merge() -> persist() -> m1;
        null() -> m1;
        m1 = merge()
            -> fold(0, |a, b| (a + b))
            -> for_each(|x| result_send.send(x).unwrap());
    };
    assert_graphvis_snapshots!(hf);

    for tick in 0..10 {
        assert_eq!(tick, hf.current_tick());
        hf.run_tick();
    }
    assert_eq!(
        &[1, 3, 6, 10, 15, 21, 28, 36, 45, 55],
        &*collect_ready::<Vec<_>, _>(&mut result_recv)
    );
}

#[multiplatform_test]
pub fn test_persist_push() {
    let (result_send, mut result_recv) = hydroflow::util::unbounded_channel::<u32>();

    let mut hf = hydroflow_syntax! {
        t0 = repeat_iter([1]) -> tee();
        t0 -> null();
        t1 = t0 -> persist() -> tee();
        t1 -> null();
        t1 -> fold(0, |a, b| (a + b)) -> for_each(|x| result_send.send(x).unwrap());
    };
    assert_graphvis_snapshots!(hf);

    for tick in 0..10 {
        assert_eq!(tick, hf.current_tick());
        hf.run_tick();
    }
    assert_eq!(
        &[1, 3, 6, 10, 15, 21, 28, 36, 45, 55],
        &*collect_ready::<Vec<_>, _>(&mut result_recv)
    );
}

#[multiplatform_test]
pub fn test_persist_join() {
    let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
    let mut flow = hydroflow::hydroflow_syntax! {
        repeat_iter([("hello", "world")]) -> [0]my_join;
        source_stream(input_recv) -> persist() -> [1]my_join;
        my_join = join::<'tick>() -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
    };
    input_send.send(("hello", "oakland")).unwrap();
    flow.run_tick();
    input_send.send(("hello", "san francisco")).unwrap();
    flow.run_tick();
}
