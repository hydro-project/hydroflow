use hydroflow::datalog;
use hydroflow::util::collect_ready;
use multiplatform_test::multiplatform_test;

#[multiplatform_test]
pub fn test_minimal() {
    let (in_send, input) = hydroflow::util::unbounded_channel::<(usize, usize)>();
    let (out, mut out_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    in_send.send((1, 2)).unwrap();

    let mut flow = datalog!(
        r#"
        .input input `source_stream(input)`
        .output out `for_each(|v| out.send(v).unwrap())`

        out(y, x) :- input(x, y).
        "#
    );

    flow.run_tick();

    assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[(2, 1)]);
}
