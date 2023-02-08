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
