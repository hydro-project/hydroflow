use hydroflow::lattices::set_union::{SetUnionHashSet, SetUnionSingletonSet, SetUnion};
use hydroflow::lattices::collections::SingletonSet;

pub fn main() {
    let mut hf = hydroflow::hydroflow_syntax! {
        source_iter_delta((0..10).map(SetUnionSingletonSet::new_from))
            -> cast(None)
            -> lattice_fold::<'static, SetUnionHashSet<_>>()
            -> cast(None)
            -> lattice_reduce::<'static>()
            -> null();
    };
    hf.run_available();

    compile_error!("warning test");
}
