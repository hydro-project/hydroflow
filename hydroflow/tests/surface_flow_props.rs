use std::error::Error;

use hydroflow::lattices::set_union::SetUnionSingletonSet;
use hydroflow::{assert_graphvis_snapshots, hydroflow_syntax};
use lattices::collections::SingletonSet;
use lattices::set_union::SetUnion;

#[test]
pub fn test_basic() -> Result<(), Box<dyn Error>> {
    let mut hf = hydroflow_syntax! {
        my_tee = source_iter_delta((0..10).map(SetUnionSingletonSet::new_from))
            -> map(|SetUnion(SingletonSet(x))| SetUnion(SingletonSet(x + 5)))
            -> tee();

        my_tee
            -> cast(None)
            -> map(|SetUnion(SingletonSet(x))| 10 * x)
            -> cast(Some(Delta))
            -> for_each(|x| println!("seq {:?}", x));

        my_tee
            -> for_each(|s| println!("delta {:?}", s));
    };
    hf.run_available();

    assert_graphvis_snapshots!(hf);

    Ok(())
}
