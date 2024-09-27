use std::collections::{HashMap, HashSet};

use hydroflow::util::collect_ready;
use hydroflow::{assert_graphvis_snapshots, hydroflow_syntax};
use lattices::ght::GeneralizedHashTrieNode;
use lattices::ght_lattice::{DeepJoinLatticeBimorphism, GhtBimorphism};
use lattices::map_union::{KeyedBimorphism, MapUnionHashMap, MapUnionSingletonMap};
use lattices::set_union::{CartesianProductBimorphism, SetUnionHashSet, SetUnionSingletonSet};
use lattices::GhtType;
use multiplatform_test::multiplatform_test;
use variadics::variadic_sets::VariadicCountedHashSet;
use variadics::{var_expr, CloneVariadic};

#[multiplatform_test]
pub fn test_cartesian_product() {
    let (out_send, out_recv) = hydroflow::util::unbounded_channel::<_>();

    let mut df = hydroflow_syntax! {
        lhs = source_iter_delta(0..3)
            -> map(SetUnionSingletonSet::new_from)
            -> state::<'static, SetUnionHashSet<u32>>();
        rhs = source_iter_delta(3..5)
            -> map(SetUnionSingletonSet::new_from)
            -> state::<'static, SetUnionHashSet<u32>>();

        lhs -> [0]my_join;
        rhs -> [1]my_join;

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

        lhs -> [0]my_join;
        rhs -> [1]my_join;

        my_join = lattice_bimorphism(KeyedBimorphism::<HashMap<_, _>, _>::new(CartesianProductBimorphism::<HashSet<_>>::default()), #lhs, #rhs)
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

/// Test for https://github.com/hydro-project/hydroflow/issues/1298
#[multiplatform_test]
pub fn test_cartesian_product_tick_state() {
    let (lhs_send, lhs_recv) = hydroflow::util::unbounded_channel::<u32>();
    let (rhs_send, rhs_recv) = hydroflow::util::unbounded_channel::<u32>();
    let (out_send, mut out_recv) = hydroflow::util::unbounded_channel::<_>();

    let mut df = hydroflow_syntax! {
        lhs = source_stream(lhs_recv)
            -> map(SetUnionSingletonSet::new_from)
            -> state::<'tick, SetUnionHashSet<u32>>();
        rhs = source_stream(rhs_recv)
            -> map(SetUnionSingletonSet::new_from)
            -> state::<'tick, SetUnionHashSet<u32>>();

        lhs[items] -> [0]my_join;
        rhs[items] -> [1]my_join;

        my_join = lattice_bimorphism(CartesianProductBimorphism::<HashSet<_>>::default(), #lhs, #rhs)
            -> lattice_reduce()
            -> inspect(|x| println!("{:?}: {:?}", context.current_tick(), x))
            -> for_each(|x| out_send.send(x).unwrap());
    };
    assert_graphvis_snapshots!(df);

    for x in 0..3 {
        lhs_send.send(x).unwrap();
    }
    for x in 3..5 {
        rhs_send.send(x).unwrap();
    }
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
        &*collect_ready::<Vec<_>, _>(&mut out_recv)
    );

    for x in 3..5 {
        lhs_send.send(x).unwrap();
    }
    df.run_available();
    assert_eq!(
        &[SetUnionHashSet::default()],
        &*collect_ready::<Vec<_>, _>(&mut out_recv)
    );
}

#[test]
fn test_ght_join_bimorphism() {
    // type MyGhtA = GhtType!(u32, u64, u16 => &'static str);
    // type MyGhtB = GhtType!(u32, u64, u16 => &'static str);
    // type MyGhtATrie = <MyGhtA as GeneralizedHashTrie>::Trie;
    // type MyGhtBTrie = <MyGhtB as GeneralizedHashTrie>::Trie;
    type MyGhtATrie = GhtType!(u32, u64, u16 => &'static str: Row);
    type MyGhtBTrie = GhtType!(u32, u64, u16 => &'static str: Row);

    type Output = variadics::var_type!(u32, u64, u16, &'static str, &'static str);

    type MyNodeBim = <(MyGhtATrie, MyGhtBTrie) as DeepJoinLatticeBimorphism<
        VariadicCountedHashSet<Output>,
    >>::DeepJoinLatticeBimorphism;
    type MyBim = GhtBimorphism<MyNodeBim>;

    let mut hf = hydroflow_syntax! {
        lhs = source_iter_delta([
                var_expr!(123, 2, 5, "hello"),
                var_expr!(50, 1, 1, "hi"),
                var_expr!(5, 1, 7, "hi"),
                var_expr!(5, 1, 7, "bye"),
            ])
            -> map(|row| MyGhtATrie::new_from([row]))
            -> state::<'tick, MyGhtATrie>();
        rhs = source_iter_delta([
                var_expr!(5, 1, 8, "hi"),
                var_expr!(5, 1, 7, "world"),
                var_expr!(5, 1, 7, "folks"),
                var_expr!(10, 1, 2, "hi"),
                var_expr!(12, 10, 98, "bye"),
            ])
            -> map(|row| MyGhtBTrie::new_from([row]))
            -> state::<'tick, MyGhtBTrie>();

        lhs[items] -> [0]my_join;
        rhs[items] -> [1]my_join;


        my_join = lattice_bimorphism(MyBim::default(), #lhs, #rhs)
            -> lattice_reduce()
            -> enumerate()
            -> inspect(|x| println!("{:?} {:#?}", context.current_tick(), x))
            -> flat_map(|(_num, ght)| ght.recursive_iter().map(<Output as CloneVariadic>::clone_var_ref).collect::<Vec<_>>())
            -> null();
            // -> for_each(|x| println!("{:#?}\n", x));
    };
    // hf.meta_graph().unwrap().open_mermaid(&Default::default());
    hf.run_available();
}
