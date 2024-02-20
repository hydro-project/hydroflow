use lattices::Max;
use multiplatform_test::multiplatform_test;

#[multiplatform_test]
pub fn test_fold_tick() {
    let mut df = hydroflow::hydroflow_syntax! {
        stream1 = source_iter(10..=30);
        stream2 = source_iter_delta(15..=25) -> map(Max::new);
        sum_of_stream2 = stream2 -> state::<Max<_>>();

        sum_of_stream2[items] -> null();
        sum_of_stream2_reference = sum_of_stream2[state];
        sum_of_stream2_reference -> null::<()>();

        // Do we need to stratify between sum_of_stream2_reference and it's usage in the filter?
        // Mark state as blocking and just move on.
        // (Little different - output of state has to be blocking)

        filtered_stream1 = stream1 -> filter(|value| {
            value <= context.state_ref(#sum_of_stream2_reference).borrow_mut().as_reveal_ref()
        }) -> for_each(|x| println!("{}", x));
    };

    df.run_available(); // Should return quickly and not hang
}
