use hydroflow::util::iter_batches_stream;
use hydroflow::{assert_graphvis_snapshots, hydroflow_syntax};
use multiplatform_test::multiplatform_test;

#[multiplatform_test]
pub fn test_flo_syntax() {
    let mut df = hydroflow_syntax! {
        users = source_iter(["alice", "bob"]);
        messages = source_stream(iter_batches_stream(0..12, 3));
        loop {
            users -> [0]cp;
            messages -> [1]cp;
            cp = cross_join::<'static, 'tick>() -> for_each(|(user, message)| println!("notify {} of {}", user, message));
        }
    };
    assert_graphvis_snapshots!(df);
    df.run_available();
}
