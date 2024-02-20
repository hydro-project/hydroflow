use std::collections::{HashMap, HashSet};

use hydroflow::util::collect_ready;
use hydroflow::{assert_graphvis_snapshots, hydroflow_syntax};
use multiplatform_test::multiplatform_test;

#[multiplatform_test]
pub fn test_fold_tick() {
    let mut df = hydroflow::hydroflow_syntax! {
        stream1 = source_iter(10..30);
        stream2 = source_iter(10..20);
        sum_of_stream2 = stream2 -> fold(|| 0, |acc, cur| *acc += cur);
        filtered_stream1 = stream1 -> filter(|value| value < #sum_of_stream2);
    };

    df.run_available(); // Should return quickly and not hang
}
