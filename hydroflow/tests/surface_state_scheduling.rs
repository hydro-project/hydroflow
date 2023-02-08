use hydroflow::hydroflow_syntax;
use hydroflow::util::collect_ready;

#[test]
pub fn test_repeat_iter() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        repeat_iter([1]) -> for_each(|v| out_send.send(v).unwrap());
    };
    assert_eq!((0, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((1, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((2, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((3, 0), (df.current_tick(), df.current_stratum()));

    assert_eq!(&[1, 1, 1], &*collect_ready::<Vec<_>, _>(&mut out_recv));
}

#[test]
pub fn test_fold_tick() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_iter([1]) -> fold::<'tick>(0, |accum, elem| accum + elem) -> for_each(|v| out_send.send(v).unwrap());
    };
    assert_eq!((0, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((1, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((2, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((3, 0), (df.current_tick(), df.current_stratum()));

    assert_eq!(&[1], &*collect_ready::<Vec<_>, _>(&mut out_recv));
}

#[test]
pub fn test_fold_static() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_iter([1]) -> fold::<'static>(0, |accum, elem| accum + elem) -> for_each(|v| out_send.send(v).unwrap());
    };
    assert_eq!((0, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((1, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((2, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((3, 0), (df.current_tick(), df.current_stratum()));

    assert_eq!(&[1, 1, 1], &*collect_ready::<Vec<_>, _>(&mut out_recv));
}

#[test]
pub fn test_reduce_tick() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_iter([1]) -> reduce::<'tick>(|a, b| a + b) -> for_each(|v| out_send.send(v).unwrap());
    };
    assert_eq!((0, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((1, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((2, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((3, 0), (df.current_tick(), df.current_stratum()));

    assert_eq!(&[1], &*collect_ready::<Vec<_>, _>(&mut out_recv));
}

#[test]
pub fn test_reduce_static() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_iter([1]) -> reduce::<'static>(|a, b| a + b) -> for_each(|v| out_send.send(v).unwrap());
    };
    assert_eq!((0, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((1, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((2, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((3, 0), (df.current_tick(), df.current_stratum()));

    assert_eq!(&[1, 1, 1], &*collect_ready::<Vec<_>, _>(&mut out_recv));
}

#[test]
pub fn test_group_by_tick() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<(char, usize)>();

    let mut df = hydroflow_syntax! {
        source_iter([('a', 1), ('a', 2)]) -> group_by::<'tick>(|| 0, |acc: &mut usize, item| { *acc += item; }) -> for_each(|v| out_send.send(v).unwrap());
    };
    assert_eq!((0, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((1, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((2, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((3, 0), (df.current_tick(), df.current_stratum()));

    assert_eq!(&[('a', 3)], &*collect_ready::<Vec<_>, _>(&mut out_recv));
}

#[test]
pub fn test_group_by_static() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<(char, usize)>();

    let mut df = hydroflow_syntax! {
        source_iter([('a', 1), ('a', 2)]) -> group_by::<'static>(|| 0, |acc: &mut usize, item| { *acc += item; }) -> for_each(|v| out_send.send(v).unwrap());
    };
    assert_eq!((0, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((1, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((2, 0), (df.current_tick(), df.current_stratum()));
    df.run_tick();
    assert_eq!((3, 0), (df.current_tick(), df.current_stratum()));

    assert_eq!(
        &[('a', 3), ('a', 3), ('a', 3)],
        &*collect_ready::<Vec<_>, _>(&mut out_recv)
    );
}
