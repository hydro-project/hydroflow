use std::error::Error;

use hydroflow::hydroflow_syntax;
use hydroflow::lattices::set_union::SetUnionSingletonSet;

#[test]
pub fn test_basic() -> Result<(), Box<dyn Error>> {
    let mut hf = hydroflow_syntax! {
        source_iter_delta((0..10).map(SetUnionSingletonSet::new_from))
            -> for_each(|x| println!("{:?}", x));
    };
    hf.run_available();

    Ok(())
}
