use std::error::Error;

use hydroflow::lattices::collections::SingletonSet;
use hydroflow::lattices::set_union::{SetUnion, SetUnionHashSet, SetUnionSingletonSet};
use hydroflow::{assert_graphvis_snapshots, hydroflow_syntax};

#[test]
pub fn test_basic() -> Result<(), Box<dyn Error>> {
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
            -> lattice_reduce::<'static, SetUnionHashSet<_>>()
            -> for_each(|s| println!("cumul {:?}", s));
    };
    hf.run_available();

    assert_graphvis_snapshots!(hf);

    Ok(())
}
