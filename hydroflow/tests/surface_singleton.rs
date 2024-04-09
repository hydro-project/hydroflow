use hydroflow::assert_graphvis_snapshots;
use lattices::Max;
use multiplatform_test::multiplatform_test;

#[multiplatform_test]
pub fn test_state_ref() {
    let mut df = hydroflow::hydroflow_syntax! {
        stream1 = source_iter(10..=30);
        stream2 = source_iter_delta(15..=25) -> map(Max::new);
        sum_of_stream2 = stream2 -> state_ref::<Max<_>>();

        filtered_stream1 = stream1
            -> inspect(|x| println!("inspect {}", x))
            -> filter(|value| {
                // This is not monotonic.
                value <= #sum_of_stream2.as_reveal_ref()
            })
            -> for_each(|x| println!("filtered {}", x));

        // Optional:
        sum_of_stream2 -> for_each(|x| println!("state {:?}", x));
    };

    assert_graphvis_snapshots!(df);

    df.run_available(); // Should return quickly and not hang
}

#[multiplatform_test]
pub fn test_state_ref_unused() {
    let mut df = hydroflow::hydroflow_syntax! {
        stream2 = source_iter_delta(15..=25) -> map(Max::new);
        sum_of_stream2 = stream2 -> state_ref::<Max<_>>();
    };

    assert_graphvis_snapshots!(df);

    df.run_available(); // Should return quickly and not hang
}

#[multiplatform_test]
pub fn test_fold_zip() {
    let mut df = hydroflow::hydroflow_syntax! {
        stream1 = source_iter(10..=30);
        stream2 = source_iter_delta(15..=25) -> map(Max::new);
        sum_of_stream2 = stream2 -> lattice_reduce() -> tee();

        filtered_stream1 = stream1
            -> inspect(|x| println!("inspect {}", x))
            -> [0]filtered_stream2;
        sum_of_stream2 -> identity::<Max<_>>() -> [1]filtered_stream2;

        filtered_stream2 = zip()
            -> filter(|(value, sum_of_stream2)| {
                // This is not monotonic.
                value <= sum_of_stream2.as_reveal_ref()
            })
            -> for_each(|x| println!("filtered {:?}", x));

        // Optional:
        sum_of_stream2 -> for_each(|x: Max<_>| println!("state {:?}", x));
    };

    assert_graphvis_snapshots!(df);

    df.run_available(); // Should return quickly and not hang
}

#[multiplatform_test]
pub fn test_fold_singleton() {
    let mut df = hydroflow::hydroflow_syntax! {
        stream1 = source_iter(10..=30);
        stream2 = source_iter(15..=25);
        sum_of_stream2 = stream2 -> fold(|| 0, std::ops::AddAssign::add_assign);

        filtered_stream1 = stream1
            -> inspect(|x| println!("inspect {}", x))
            -> filter(|&value| {
                // This is not monotonic.
                value <= #sum_of_stream2
            })
            -> for_each(|x| println!("filtered {}", x));

        sum_of_stream2 -> for_each(|x| println!("state {:?}", x));
    };

    assert_graphvis_snapshots!(df);

    df.run_available();
}

#[multiplatform_test]
pub fn test_fold_singleton_push() {
    let mut df = hydroflow::hydroflow_syntax! {
        stream1 = source_iter(10..=30);
        stream2 = source_iter(15..=25);
        sum_of_stream2 = stream2 -> fold(|| 0, std::ops::AddAssign::add_assign);

        filtered_stream1 = stream1
            -> inspect(|x| println!("inspect {}", x))
            -> filter(|&value| {
                // This is not monotonic.
                value <= #sum_of_stream2
            })
            -> for_each(|x| println!("filtered {}", x));
    };

    assert_graphvis_snapshots!(df);

    df.run_available();
}
