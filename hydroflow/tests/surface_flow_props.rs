use hydroflow::lattices::collections::SingletonSet;
use hydroflow::lattices::set_union::{SetUnion, SetUnionHashSet, SetUnionSingletonSet};
use hydroflow::{assert_graphvis_snapshots, hydroflow_expect_warnings, hydroflow_syntax};

#[test]
pub fn test_basic() {
    let mut hf = hydroflow_syntax! {
        my_tee = source_iter_delta((0..10).map(SetUnionSingletonSet::new_from))
            -> map(|SetUnion(SingletonSet(x))| SetUnion(SingletonSet(x + 5)))
            -> tee();

        my_tee
            -> cast(None)
            -> map(|SetUnion(SingletonSet(x))| 10 * x)
            -> for_each(|x| println!("seq {:?}", x));

        my_tee
            -> for_each(|s| println!("delta {:?}", s));

        my_tee
            -> map(|SetUnion(SingletonSet(x))| SetUnionHashSet::new_from([x]))
            -> lattice_reduce()
            -> for_each(|s| println!("cumul {:?}", s));
    };
    hf.run_available();

    assert_graphvis_snapshots!(hf);
}

#[test]
pub fn test_union_warning() {
    let mut hf = hydroflow_expect_warnings! {
        {
            source_iter_delta((0..10).map(SetUnionSingletonSet::new_from)) -> [0]my_union;
            source_iter((0..10).map(SetUnionSingletonSet::new_from)) -> [1]my_union;

            my_union = union() -> for_each(|s| println!("{:?}", s));
        },
        "Warning: Input to `union()` will be downcast to `None` to match other inputs.\n  --> $FILE:0:0"
    };
    hf.run_available();

    assert_graphvis_snapshots!(hf);
}
