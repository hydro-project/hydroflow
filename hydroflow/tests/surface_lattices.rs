use std::collections::HashMap;

use hydroflow::lang::lattice::map_union::MapUnionRepr;
use hydroflow::lang::lattice::ord::MaxRepr;
use hydroflow::lang::lattice::Merge;
use hydroflow::lang::tag;

#[test]
fn surface_lattice_group_by() {
    type VecClock = MapUnionRepr<tag::HASH_MAP, &'static str, MaxRepr<u64>>;

    let mut hf = hydroflow::hydroflow_syntax! {
        source_iter([
                [("us-west", 5), ("us-east", 12)],
                [("jp", 2), ("us-west", 4)],
                [("jp", 3), ("us-west", 7)],
            ])
            -> map(|pairs| pairs.into_iter().collect::<HashMap<&str, u64>>())
            // -> lub::<VecClock>()
            -> fold(Default::default(), <VecClock as Merge<VecClock>>::fold)
            -> for_each(|pair| println!("{:?}", pair));
    };
    hf.run_available();
}
