use std::collections::{HashMap, HashSet};

use hydroflow::util::collect_ready;
use hydroflow::{assert_graphvis_snapshots, hydroflow_syntax};
use lattices::Max;
use multiplatform_test::multiplatform_test;

#[multiplatform_test]
pub fn test_fold_tick() {
    let mut df = hydroflow::hydroflow_syntax! {
        stream1 = source_iter(10..30);
        stream2 = source_iter_delta(10..20) -> map(Max::new);
        sum_of_stream2 = stream2 -> state::<Max<_>>();
        sum_of_stream2[items] -> null();
        sum_of_stream2_reference = sum_of_stream2[state];
        sum_of_stream2_reference -> null();
        filtered_stream1 = stream1 -> filter(|value| value < #sum_of_stream2_reference) -> for_each(|x| println!("{}", x));
    };

    df.run_available(); // Should return quickly and not hang
}
