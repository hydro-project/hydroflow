use multiplatform_test::multiplatform_test;

use hydroflow::{assert_graphvis_snapshots, hydroflow_syntax};

#[multiplatform_test]
pub fn test_join_empty_lhs() {
    let (result_send, _) = hydroflow::util::unbounded_channel::<(u32, u32)>();

    let mut hf = hydroflow_syntax! {
        join_node = join::<'tick, 'tick>() -> for_each(|x: (u32, (u32, u32))| result_send.send(x.1).unwrap());
        null() -> [0] join_node;
        repeat_iter(1..) -> map(|x| (x, x)) -> [1] join_node;
    };
    assert_graphvis_snapshots!(hf);

    for tick in 0..10 {
        assert_eq!(tick, hf.current_tick());
        hf.run_tick();
    }
}

#[multiplatform_test]
pub fn test_join_empty_rhs() {
    let (result_send, _) = hydroflow::util::unbounded_channel::<(u32, u32)>();

    let mut hf = hydroflow_syntax! {
        join_node = join::<'tick, 'tick>() -> for_each(|x: (u32, (u32, u32))| result_send.send(x.1).unwrap());
        repeat_iter(1..) -> map(|x| (x, x)) -> [0] join_node;
        null() -> [1] join_node;
    };
    assert_graphvis_snapshots!(hf);

    for tick in 0..10 {
        assert_eq!(tick, hf.current_tick());
        hf.run_tick();
    }
}
