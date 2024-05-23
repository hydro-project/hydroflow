use std::collections::{HashMap, HashSet};

use hydroflow::util::collect_ready;
use hydroflow::{assert_graphvis_snapshots, hydroflow_syntax};
use lattices::map_union::{KeyedBimorphism, MapUnionHashMap, MapUnionSingletonMap};
use lattices::set_union::{CartesianProductBimorphism, SetUnionHashSet, SetUnionSingletonSet};
use multiplatform_test::multiplatform_test;

#[multiplatform_test]
pub fn test_cartesian_product() {
    let (out_send, out_recv) = hydroflow::util::unbounded_channel::<_>();

    let mut df = hydroflow_syntax! {
        lhs = source_iter_delta(0..3)
            -> map(SetUnionSingletonSet::new_from)
            -> state::<'static, SetUnionHashSet<usize>>();
        rhs = source_iter_delta(3..5)
            -> map(SetUnionSingletonSet::new_from)
            -> state::<'static, SetUnionHashSet<usize>>();

        lhs[items] -> [0]my_join;
        rhs[items] -> [1]my_join;

        my_join = lattice_bimorphism(CartesianProductBimorphism::<HashSet<_>>::default(), #lhs, #rhs)
            -> lattice_reduce()
            -> for_each(|x| out_send.send(x).unwrap());
    };

    assert_graphvis_snapshots!(df);
    df.run_available();

    assert_eq!(
        &[SetUnionHashSet::new(HashSet::from_iter([
            (0, 3),
            (0, 4),
            (1, 3),
            (1, 4),
            (2, 3),
            (2, 4),
        ]))],
        &*collect_ready::<Vec<_>, _>(out_recv)
    );
}

#[multiplatform_test]
pub fn test_join() {
    let (out_send, out_recv) = hydroflow::util::unbounded_channel::<_>();

    let mut df = hydroflow_syntax! {
        lhs = source_iter_delta([(7, 1), (7, 2)])
            -> map(|(k, v)| MapUnionSingletonMap::new_from((k, SetUnionSingletonSet::new_from(v))))
            -> state::<'static, MapUnionHashMap<usize, SetUnionHashSet<usize>>>();
        rhs = source_iter_delta([(7, 0), (7, 1), (7, 2)])
            -> map(|(k, v)| MapUnionSingletonMap::new_from((k, SetUnionSingletonSet::new_from(v))))
            -> state::<'static, MapUnionHashMap<usize, SetUnionHashSet<usize>>>();

        lhs[items] -> [0]my_join;
        rhs[items] -> [1]my_join;

        my_join = lattice_bimorphism(KeyedBimorphism::<HashMap<_, _>, _>::from(CartesianProductBimorphism::<HashSet<_>>::default()), #lhs, #rhs)
            -> lattice_reduce()
            -> for_each(|x| out_send.send(x).unwrap());
    };

    assert_graphvis_snapshots!(df);
    df.run_available();

    assert_eq!(
        &[MapUnionHashMap::new(HashMap::from_iter([(
            7,
            SetUnionHashSet::new(HashSet::from_iter([
                (1, 0),
                (1, 1),
                (1, 2),
                (2, 0),
                (2, 1),
                (2, 2),
            ]))
        )]))],
        &*collect_ready::<Vec<_>, _>(out_recv)
    );
}
