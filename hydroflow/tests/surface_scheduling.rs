use hydroflow::hydroflow_syntax;
use multiplatform_test::multiplatform_test;

#[multiplatform_test(test, wasm, env_tracing)]
pub fn test_stratum_loop() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_iter([0]) -> union_tee;
        union_tee = union() -> tee();
        union_tee -> map(|n| n + 1) -> filter(|&n| n < 10) -> next_stratum() -> union_tee;
        union_tee -> for_each(|v| out_send.send(v).unwrap());
    };
    df.run_available();

    assert_eq!(
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        &*hydroflow::util::collect_ready::<Vec<_>, _>(&mut out_recv)
    );
    assert_eq!((11, 0), (df.current_tick(), df.current_stratum()));
}

#[multiplatform_test(test, wasm, env_tracing)]
pub fn test_tick_loop() {
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<usize>();

    let mut df = hydroflow_syntax! {
        source_iter([0]) -> union_tee;
        union_tee = union() -> tee();
        union_tee -> map(|n| n + 1) -> filter(|&n| n < 10) -> next_tick() -> union_tee;
        union_tee -> for_each(|v| out_send.send(v).unwrap());
    };
    df.run_available();

    assert_eq!(
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        &*hydroflow::util::collect_ready::<Vec<_>, _>(&mut out_recv)
    );
    assert_eq!((10, 0), (df.current_tick(), df.current_stratum()));
}
