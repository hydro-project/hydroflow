use hydroflow::assert_graphvis_snapshots;
use hydroflow::util::collect_ready;
use multiplatform_test::multiplatform_test;

#[multiplatform_test]
pub fn test_multiset_delta() {
    let (input_send, input_recv) = hydroflow::util::unbounded_channel::<u32>();
    let (result_send, mut result_recv) = hydroflow::util::unbounded_channel::<u32>();

    let mut flow = hydroflow::hydroflow_syntax! {
        source_stream(input_recv)
            -> multiset_delta()
            -> for_each(|x| result_send.send(x).unwrap());
    };
    assert_graphvis_snapshots!(flow);

    input_send.send(3).unwrap();
    input_send.send(4).unwrap();
    input_send.send(3).unwrap();
    flow.run_tick();
    assert_eq!(&[3, 4, 3], &*collect_ready::<Vec<_>, _>(&mut result_recv));

    input_send.send(3).unwrap();
    input_send.send(5).unwrap();
    input_send.send(3).unwrap();
    input_send.send(3).unwrap();
    flow.run_tick();
    // First two "3"s are removed due to previous tick.
    assert_eq!(&[5, 3], &*collect_ready::<Vec<_>, _>(&mut result_recv));
}
