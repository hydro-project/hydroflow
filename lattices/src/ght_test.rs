#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::io::{self, Write};

    use variadics::variadic_collections::{
        VariadicColumnMultiset, VariadicCountedHashSet, VariadicMultiset,
    };
    use variadics::{var_expr, var_type, VariadicExt};

    use crate::ght::{GeneralizedHashTrieNode, GhtGet, GhtLeaf, GhtPrefixIter};
    use crate::ght_lattice::{
        //     DeepJoinLatticeBimorphism, GhtBimorphism,
        GhtCartesianProductBimorphism,
        //     GhtNodeKeyedBimorphism, GhtValTypeProductBimorphism,
    };
    use crate::ght_lazy::{ColtNode, ColumnLazyTrieNode, ForestFindLeaf, GhtForest}; /* GhtForestStruct}; */
    use crate::{GhtForestType, GhtType, LatticeBimorphism, Merge, NaiveLatticeOrd};

    #[test]
    fn basic_test() {
        // Example usage
        type MyTrie1 = GhtType!(u32, u32 => &'static str: Row);

        fn ght_type<T: GeneralizedHashTrieNode>() {}
        ght_type::<MyTrie1>();

        let htrie1 = MyTrie1::new_from(vec![var_expr!(42, 314, "hello")]);
        assert!(htrie1.contains(var_expr!(&42, &314, &"hello")));
        assert_eq!(htrie1.recursive_iter().count(), 1);

        type MyTrie2 = GhtType!(u32 => u32: Row);
        let htrie2 = MyTrie2::new_from(vec![var_expr!(42, 314)]);
        assert!(htrie2.contains(var_expr!(&42, &314)));
        assert_eq!(htrie1.recursive_iter().count(), 1);

        type MyTrie3 = GhtType!(u32, u64, u16 => &'static str: Row);
        let htrie3 = MyTrie3::new_from(vec![
            var_expr!(123, 2, 5, "hello"),
            var_expr!(50, 1, 1, "hi"),
            var_expr!(5, 1, 7, "hi"),
            var_expr!(5, 1, 7, "bye"),
        ]);
        assert!(htrie3.contains(var_expr!(&50, &1, &1, &"hi")));
        assert_eq!(htrie3.recursive_iter().count(), 4);
    }
    #[test]
    fn test_ght_node_type_macro() {
        type LilTrie = GhtType!(() => u32: Row);
        let _j = LilTrie::default();
        let _l = LilTrie::new_from(vec![var_expr!(1)]);

        type LilTrie2 = GhtType!(() => u32, u64: Row);
        let _l = LilTrie2::default();
        let _l = LilTrie2::new_from(vec![var_expr!(1, 1)]);

        type SmallTrie = GhtType!(u32 => &'static str: Row);
        type SmallKeyedTrie = GhtType!(u32 => &'static str: Row);
        let l = SmallTrie::new_from(vec![var_expr!(1, "hello")]);
        let _: SmallKeyedTrie = l;

        type LongKeySmallValTrie = GhtType!(u32, u16 => &'static str: Row);
        type LongKeySmallValKeyedTrie = GhtType!(u32, u16 => &'static str: Row);
        let x = LongKeySmallValTrie::new_from(vec![var_expr!(1, 314, "hello")]);
        let _: LongKeySmallValKeyedTrie = x;
        let _ = LongKeySmallValTrie::new_from(vec![var_expr!(1, 314, "hello")]);

        type SmallKeyLongValTrie = GhtType!(u32 => u64, u16, &'static str: Row);
        let _x = SmallKeyLongValTrie::new_from(vec![var_expr!(1, 999, 222, "hello")]);

        type LongKeyLongValTrie = GhtType!(u32, u64 => u16, &'static str: Row);
        let _x = LongKeyLongValTrie::new_from(vec![var_expr!(1, 999, 222, "hello")]);
    }

    #[test]
    fn test_insert() {
        let mut htrie = <GhtType!(u16, u32 => u64: Row)>::default();
        htrie.insert(var_expr!(42, 314, 43770));
        assert_eq!(htrie.recursive_iter().count(), 1);
        assert_eq!(htrie.height(), 2);
        htrie.insert(var_expr!(42, 315, 43770));
        assert_eq!(htrie.recursive_iter().count(), 2);
        assert_eq!(htrie.height(), 2);
        htrie.insert(var_expr!(42, 314, 30619));
        assert_eq!(htrie.recursive_iter().count(), 3);
        assert_eq!(htrie.height(), 2);
        htrie.insert(var_expr!(43, 10, 600));
        assert_eq!(htrie.recursive_iter().count(), 4);
        assert_eq!(htrie.height(), 2);
        assert!(htrie.contains(var_expr!(&42, &314, &30619)));
        assert!(htrie.contains(var_expr!(&42, &315, &43770)));
        assert!(htrie.contains(var_expr!(&43, &10, &600)));

        type LongKeyLongValTrie = GhtType!(u32, u64 => u16, &'static str: Row);
        let mut htrie = LongKeyLongValTrie::new_from(vec![var_expr!(1, 999, 222, "hello")]);
        htrie.insert(var_expr!(1, 999, 111, "bye"));
        assert_eq!(htrie.height(), 2);
        htrie.insert(var_expr!(1, 1000, 123, "cya"));
        assert_eq!(htrie.height(), 2);
        // println!("htrie: {:?}", htrie);
        assert!(htrie.contains(var_expr!(&1, &999, &222, &"hello")));
        assert!(htrie.contains(var_expr!(&1, &999, &111, &"bye")));
        assert!(htrie.contains(var_expr!(&1, &1000, &123, &"cya")));
    }

    #[test]
    fn test_scale() {
        type MyGht = GhtType!(bool, usize, &'static str => i32: Row);
        let mut htrie = MyGht::new_from(vec![var_expr!(true, 1, "hello", -5)]);
        assert_eq!(htrie.recursive_iter().count(), 1);
        for i in 1..1000000 {
            htrie.insert(var_expr!(true, 1, "hello", i));
        }
        assert_eq!(htrie.recursive_iter().count(), 1000000);
    }

    #[test]
    fn test_contains() {
        type MyGht = GhtType!(u16, u32 => u64: Row);
        let htrie = MyGht::new_from(vec![var_expr!(42_u16, 314_u32, 43770_u64)]);
        // println!("HTrie: {:?}", htrie);
        let x = var_expr!(&42, &314, &43770);
        assert!(htrie.contains(x));
        assert!(htrie.contains(var_expr!(42, 314, 43770).as_ref_var()));
        assert!(htrie.contains(var_expr!(&42, &314, &43770)));
        assert!(!htrie.contains(var_expr!(42, 314, 30619).as_ref_var()));
        assert!(!htrie.contains(var_expr!(&42, &315, &43770)));
        assert!(!htrie.contains(var_expr!(&43, &314, &43770)));
    }

    #[test]
    fn test_get() {
        type MyGht = GhtType!(u32, u32 => u32: Row);
        let ht_root = MyGht::new_from(vec![var_expr!(42, 314, 43770)]);

        let inner = ht_root.get(&42).unwrap();
        let t = inner.recursive_iter().next().unwrap();
        assert_eq!(t, var_expr!(&42, &314, &43770));

        let leaf = inner.get(&314).unwrap();
        let t = leaf.recursive_iter().next().unwrap();
        assert_eq!(t, var_expr!(42, 314, 43770).as_ref_var());
    }

    // #[test]
    // fn test_iter() {
    //     type MyGht = GhtType!(u32, u32 => u32);
    //     let ht_root = MyGht::new_from(vec![var_expr!(42, 314, 43770)]);
    //     let inner_key = ht_root.iter().next().unwrap();
    //     let inner = ht_root.get(inner_key).unwrap();
    //     let t = inner.recursive_iter().next().unwrap();
    //     assert_eq!(t, var_expr!(&42, &314, &43770));

    //     let leaf_key = inner.iter().next().unwrap();
    //     let leaf = inner.get(GhtHead::Head(leaf_key)).unwrap();
    //     // Should not be possible to call iter() on leaf
    //     // let t = leaf.iter().next();
    //     // assert!(leaf.iter().next().is_none());
    // }

    #[test]
    fn test_recursive_iter() {
        type MyGht = GhtType!(u32, u32 => u32: Row);
        type InputType = var_type!(u32, u32, u32);
        type ResultType<'a> = var_type!(&'a u32, &'a u32, &'a u32);
        let input: HashSet<InputType> = HashSet::from_iter(
            [
                (42, 314, 30619),
                (42, 314, 43770),
                (42, 315, 43770),
                (43, 10, 600),
            ]
            .iter()
            .map(|&(a, b, c)| var_expr!(a, b, c)),
        );
        let htrie = MyGht::new_from(input.clone());
        let result = input.iter().map(|v| v.as_ref_var()).collect();
        let v: HashSet<ResultType> = htrie.recursive_iter().collect();
        assert_eq!(v, result);
    }

    #[test]
    fn test_prefix_iter_leaf() {
        use crate::ght::GhtPrefixIter;
        type InputType = var_type!(u8, u16, u32);
        type ResultType<'a> = var_type!(&'a u8, &'a u16, &'a u32);

        let input: HashSet<InputType> = HashSet::from_iter(
            [
                (42, 314, 30619),
                (42, 314, 43770),
                (42, 315, 43770),
                (43, 10, 600),
            ]
            .iter()
            .map(|&(a, b, c)| var_expr!(a, b, c)),
        );
        let leaf =
            GhtLeaf::<InputType, var_type!(u16, u32), VariadicCountedHashSet<InputType>>::new_from(
                input.clone(),
            );
        // let key = var_expr!(42u8).as_ref_var();
        let key = (); // (var_expr!().as_ref_var();)
        let v: HashSet<ResultType> = leaf.prefix_iter(key).collect();
        let result = input
            .iter()
            // .filter(|t: &&InputType| t.0 == 42)
            .map(|t: &InputType| var_expr!(&t.0, &t.1 .0, &t.1 .1 .0))
            .collect();
        assert_eq!(v, result);
    }

    #[test]
    fn test_prefix_iter() {
        type MyGht = GhtType!(u8, u16 => u32: Row);
        type InputType = var_type!(u8, u16, u32);
        type ResultType<'a> = var_type!(&'a u8, &'a u16, &'a u32);
        let input: HashSet<InputType> = HashSet::from_iter(
            [
                (42, 314, 30619),
                (42, 314, 43770),
                (42, 315, 43770),
                (43, 10, 600),
            ]
            .iter()
            .map(|&(a, b, c)| var_expr!(a, b, c)),
        );
        let htrie = MyGht::new_from(input.clone());

        let v: HashSet<ResultType> = htrie.prefix_iter(var_expr!(42, 315).as_ref_var()).collect();
        let result = HashSet::from_iter([var_expr!(&42, &315, &43770)].iter().copied());
        assert_eq!(v, result);

        let v: HashSet<ResultType> = htrie.prefix_iter(var_expr!(42u8).as_ref_var()).collect();
        let result = input
            .iter()
            .filter(|t: &&InputType| t.0 == 42)
            .map(|t: &InputType| var_expr!(&t.0, &t.1 .0, &t.1 .1 .0))
            .collect();
        assert_eq!(v, result);

        // // Too long:
        // for row in htrie.prefix_iter(var_expr!(42, 315, 43770).as_ref_var()) {
        //     println!("42,315,43770: {:?}", row);
        // }
    }

    // #[test]
    // fn test_find_containing_leaf() {
    //     type MyGht = GhtType!(u32, u32 => u32);
    //     type InputType = var_type!(u32, u32, u32);
    //     let input: HashSet<InputType> = HashSet::from_iter(
    //         [
    //             (42, 314, 30619),
    //             (42, 314, 43770),
    //             (42, 315, 43770),
    //             (43, 10, 600),
    //         ]
    //         .iter()
    //         .map(|&(a, b, c)| var_expr!(a, b, c)),
    //     );
    //     let htrie = MyGht::new_from(input.clone());

    //     assert!(
    //         FindLeaf::find_containing_leaf(&htrie, var_expr!(42u32, 315u32, 43770u32)).is_some(),
    //     );
    // }

    #[test]
    fn test_prefix_iter_complex() {
        type MyGht = GhtType!(bool, u32, &'static str => i32: Row);
        type InputType = var_type!(bool, u32, &'static str, i32);
        type ResultType<'a> = var_type!(&'a bool, &'a u32, &'a &'static str, &'a i32);
        let input: HashSet<InputType> = HashSet::from_iter(
            [
                (true, 1, "hello", -5),
                (true, 1, "hi", -2),
                (true, 1, "hi", -3),
                (true, 1, "hi", -4),
                (true, 1, "hi", -5),
                (true, 2, "hello", 1),
                (false, 10, "bye", 5),
            ]
            .iter()
            .map(|&(a, b, c, d)| var_expr!(a, b, c, d)),
        );

        let htrie = MyGht::new_from(input.clone());

        let v: HashSet<ResultType> = htrie
            .prefix_iter(var_expr!(true, 1, "hi").as_ref_var())
            .collect();
        let result = input
            .iter()
            .filter(|t: &&InputType| t.0 && t.1.0 == 1 && t.1.1.0 == "hi")
            //.map(|t: &InputType| (&t.0, &t.1 .0, (&t.1 .1 .0, (&t.1 .1 .1 .0, ()))))
            .map(|t| t.as_ref_var())
            .collect();
        assert_eq!(v, result);

        let v: HashSet<ResultType> = htrie.prefix_iter(var_expr!(true).as_ref_var()).collect();
        let result = input
            .iter()
            .filter(|t: &&InputType| t.0)
            .map(|t: &InputType| t.as_ref_var())
            .collect();
        assert_eq!(v, result);
    }
    #[test]
    fn test_merge() {
        type MyGht = GhtType!(u32, u64 => u16, &'static str: Set);

        let mut test_ght1 = MyGht::new_from(vec![var_expr!(42, 314, 10, "hello")]);
        let test_ght2 = MyGht::new_from(vec![var_expr!(42, 314, 10, "hello")]);

        // merge contains duplicate copy of the tuple
        assert_eq!(
            test_ght1
                .recursive_iter()
                .collect::<Vec<var_type!(&u32, &u64, &u16, &&'static str)>>()
                .len(),
            1
        );
        test_ght1.merge(test_ght2.clone());
        assert_eq!(
            test_ght1
                .recursive_iter()
                .collect::<Vec<var_type!(&u32, &u64, &u16, &&'static str)>>()
                .len(),
            2
        );
        let mut iter = test_ght1.recursive_iter();
        assert_eq!(iter.next(), iter.next());
        // assert!(!test_ght1.merge(test_ght2.clone()));

        let mut test_ght1 = MyGht::new_from(vec![var_expr!(42, 314, 10, "hello")]);
        let mut test_ght2 = MyGht::new_from(vec![var_expr!(42, 314, 10, "hello")]);
        test_ght1.merge(test_ght2.clone());

        test_ght1.insert(var_expr!(42, 314, 20, "goodbye"));
        test_ght2.insert(var_expr!(42, 314, 20, "again"));

        // change on merge
        assert!(test_ght1.merge(test_ght2.clone()));
        for k in test_ght2.recursive_iter() {
            assert!(test_ght1.contains(k))
        }
    }
    #[test]
    fn test_node_lattice() {
        type MyGht = GhtType!(u32, u64 => u16, &'static str: Set);
        type MyGhtNode = GhtType!(u32, u64 => u16, &'static str: Set);

        let mut test_vec: Vec<MyGhtNode> = Vec::new();

        let empty_ght = MyGht::new_from(vec![]);
        let test_ght1 = MyGht::new_from(vec![var_expr!(42, 314, 10, "hello")]);
        let mut test_ght2 = MyGht::new_from(vec![var_expr!(42, 314, 10, "hello")]);
        test_ght2.insert(var_expr!(42, 314, 20, "again"));
        let mut test_ght3 = test_ght2.clone();
        test_ght3.insert(var_expr!(42, 400, 1, "level 2"));
        let mut test_ght4 = test_ght3.clone();
        test_ght4.insert(var_expr!(43, 1, 1, "level 1"));

        let test_vec_wrap = [empty_ght, test_ght1, test_ght2, test_ght3, test_ght4];

        for ght in test_vec_wrap.iter().cloned() {
            ght.naive_cmp(&ght.clone());
            test_vec.push(ght);
        }
        crate::test::check_all(&test_vec);
        crate::test::check_all(&test_vec_wrap);
    }

    #[test]
    fn test_cartesian_bimorphism() {
        type MyGhtA = GhtType!(u32, u64 => u16, &'static str: Set);
        type MyGhtB = GhtType!(u32, u64, u16 => &'static str: Set);

        let mut ght_a = MyGhtA::default();
        let mut ght_b = MyGhtB::default();

        ght_a.insert(var_expr!(123, 2, 5, "hello"));
        ght_a.insert(var_expr!(50, 1, 1, "hi"));
        ght_a.insert(var_expr!(5, 1, 7, "hi"));
        ght_b.insert(var_expr!(5, 1, 8, "hi"));
        ght_b.insert(var_expr!(10, 1, 2, "hi"));
        ght_b.insert(var_expr!(12, 10, 98, "bye"));

        type MyGhtAb = GhtType!(u32, u64, u16, &'static str, u32, u64 => u16, &'static str: Row);

        let mut bim = GhtCartesianProductBimorphism::<MyGhtAb>::default();
        let ght_out = bim.call(&ght_a, &ght_b);
        assert_eq!(
            ght_out.recursive_iter().count(),
            ght_a.recursive_iter().count() * ght_b.recursive_iter().count()
        );
    }

    // #[test]
    // fn test_join_bimorphism() {
    //     type MyGhtATrie = GhtType!(u32, u64, u16 => &'static str);
    //     type MyGhtBTrie = GhtType!(u32, u64, u16 => &'static str);

    //     let mut ght_a = MyGhtATrie::default();
    //     let mut ght_b = MyGhtBTrie::default();

    //     ght_a.insert(var_expr!(123, 2, 5, "hello"));
    //     ght_a.insert(var_expr!(50, 1, 1, "hi"));
    //     ght_a.insert(var_expr!(5, 1, 7, "hi"));

    //     ght_b.insert(var_expr!(5, 1, 8, "hi"));
    //     ght_b.insert(var_expr!(5, 1, 7, "world"));
    //     ght_b.insert(var_expr!(10, 1, 2, "hi"));
    //     ght_b.insert(var_expr!(12, 10, 98, "bye"));

    //     {
    //         type MyResultType<'a> = var_type!(
    //             &'a u32,
    //             &'a u64,
    //             &'a u16,
    //             &'a &'static str,
    //             &'a u64,
    //             &'a u16,
    //             &'a &'static str
    //         );
    //         let result: HashSet<MyResultType> = [
    //             var_expr!(&5, &1, &7, &"hi", &1, &7, &"world"),
    //             var_expr!(&5, &1, &7, &"hi", &1, &8, &"hi"),
    //         ]
    //         .iter()
    //         .copied()
    //         .collect();
    //         // type CartProdOld = GhtType!(() => u64, u16, &'static str, u64, u16, &'static str);
    //         type CartProd = GhtLeaf<
    //             var_type!(u32, u64, u16, &'static str, u64, u16, &'static str),
    //             var_type!(),
    //         >;
    //         // let _cp = ValTypeProd::default();
    //         let mut bim =
    //             GhtNodeKeyedBimorphism::new(GhtCartesianProductBimorphism::<CartProd>::default());
    //         let out = LatticeBimorphism::call(&mut bim, &ght_a, &ght_b);
    //         let out: HashSet<MyResultType> = out.recursive_iter().collect();
    //         assert_eq!(out, result.iter().copied().collect());

    //         // var!(a, b, c, d)
    //         // var!(a, b, c, d)
    //         // out: (a, b, c, d, b, c, d)
    //         // GhtNodeKeyedBimorphism<GhtCartesianProductBimorphism<Ght<var!(b, c, d, b, c, d)>>
    //         //
    //         // So what does GhtCartesianProduct bimorphism need to do?
    //         //
    //     }

    //     {
    //         type MyResultType<'a> = var_type!(
    //             &'a u32,
    //             &'a u64,
    //             &'a u16,
    //             &'a &'static str,
    //             &'a u16,
    //             &'a &'static str
    //         );
    //         let result: HashSet<MyResultType> = [
    //             var_expr!(&5, &1, &7, &"hi", &7, &"world"),
    //             var_expr!(&5, &1, &7, &"hi", &8, &"hi"),
    //         ]
    //         .iter()
    //         .copied()
    //         .collect();
    //         // type CartProd = GhtType!(u16, &'static str, u16 => &'static str);
    //         type CartProd = GhtInner<
    //             u16,
    //             GhtInner<
    //                 &'static str,
    //                 GhtInner<
    //                     u16,
    //                     GhtLeaf<
    //                         var_type!(u32, u64, u16, &'static str, u16, &'static str),
    //                         var_type!(&'static str),
    //                     >,
    //                 >,
    //             >,
    //         >;
    //         let mut bim = GhtNodeKeyedBimorphism::new(GhtNodeKeyedBimorphism::new(
    //             GhtValTypeProductBimorphism::<CartProd>::default(),
    //         ));
    //         let out = LatticeBimorphism::call(&mut bim, &ght_a, &ght_b);
    //         let out: HashSet<MyResultType> = out.recursive_iter().collect();
    //         assert_eq!(out, result.iter().copied().collect());
    //     }

    //     {
    //         type MyResultType<'a> = var_type!(
    //             &'a u32,
    //             &'a u64,
    //             &'a u16,
    //             &'a &'static str,
    //             &'a &'static str
    //         );
    //         let result: HashSet<MyResultType> = [var_expr!(&5, &1, &7, &"hi", &"world")]
    //             .iter()
    //             .copied()
    //             .collect();
    //         // type MyGhtOut = GhtType!(&'static str => &'static str);
    //         type MyGhtOut = GhtInner<
    //             &'static str,
    //             GhtLeaf<
    //                 var_type!(u32, u64, u16, &'static str, &'static str),
    //                 var_type!(&'static str),
    //             >,
    //         >;
    //         let mut bim = GhtNodeKeyedBimorphism::new(GhtNodeKeyedBimorphism::new(
    //             GhtNodeKeyedBimorphism::new(GhtValTypeProductBimorphism::<MyGhtOut>::default()),
    //         ));
    //         let out = bim.call(&ght_a, &ght_b);
    //         let out: HashSet<MyResultType> = out.recursive_iter().collect();
    //         assert_eq!(out, result.iter().copied().collect());
    //     }
    //     {
    //         // This is a more compact representation of the block above.
    //         type MyResultType<'a> = var_type!(
    //             &'a u32,
    //             &'a u64,
    //             &'a u16,
    //             &'a &'static str,
    //             &'a &'static str
    //         );
    //         let result: HashSet<MyResultType> = [var_expr!(&5, &1, &7, &"hi", &"world")]
    //             .iter()
    //             .copied()
    //             .collect();
    //         type MyNodeBim =
    //             <(MyGhtATrie, MyGhtBTrie) as DeepJoinLatticeBimorphism>::DeepJoinLatticeBimorphism;
    //         let mut bim = <MyNodeBim as Default>::default();
    //         // let out = <MyNodeBim as LatticeBimorphism<&MyGhtATrie, &MyGhtBTrie>>::call(
    //         //     &mut bim, &ght_a, &ght_b,
    //         // );
    //         let out = bim.call(&ght_a, &ght_b);
    //         let out: HashSet<MyResultType> = out.recursive_iter().collect();
    //         assert_eq!(out, result.iter().copied().collect());
    //     }
    //     {
    //         // This is an even more compact representation of the block above.
    //         type MyResultType<'a> = var_type!(
    //             &'a u32,
    //             &'a u64,
    //             &'a u16,
    //             &'a &'static str,
    //             &'a &'static str
    //         );
    //         let result: HashSet<MyResultType> = [var_expr!(&5, &1, &7, &"hi", &"world")]
    //             .iter()
    //             .copied()
    //             .collect();
    //         type MyNodeBim = <MyGhtATrie as GeneralizedHashTrieNode>::DeepJoin<MyGhtBTrie>;
    //         let mut bim = <MyNodeBim as Default>::default();
    //         // let out = <MyNodeBim as LatticeBimorphism<&MyGhtATrie, &MyGhtBTrie>>::call(
    //         //     &mut bim, &ght_a, &ght_b,
    //         // );
    //         let out = bim.call(&ght_a, &ght_b);
    //         let out: HashSet<MyResultType> = out.recursive_iter().collect();
    //         assert_eq!(out, result.iter().copied().collect());
    //     }
    // }

    // #[test]
    // fn test_recursive_iter_keys() {
    //     type MyGht = GhtType!(u32 => u32, u32);
    //     type InputType = var_type!(u32, u32, u32);
    //     type OutputType = var_type!(u32);
    //     type ResultType<'a> = var_type!(&'a u32);
    //     let input: HashSet<InputType> = HashSet::from_iter(
    //         [
    //             (42, 314, 43770),
    //             (42, 315, 43770),
    //             (43, 10, 600),
    //             (43, 10, 60),
    //         ]
    //         .iter()
    //         .map(|&(a, b, c)| var_expr!(a, b, c)),
    //     );
    //     let output: HashSet<OutputType> = [var_expr!(42), var_expr!(43)].iter().copied().collect();

    //     let htrie = MyGht::new_from(input.clone());
    //     let result = output.iter().map(|v| v.as_ref_var()).collect();
    //     let v: HashSet<ResultType> = htrie
    //         .recursive_iter_keys()
    //         .map(|(a, (_b, (_c, ())))| var_expr!(a))
    //         .collect();
    //     assert_eq!(v, result);
    // }

    pub trait PartialEqExceptNested: variadics::Variadic {
        fn eq_except_nested(&self, other: &Self) -> bool;
    }

    // Implement the trait for the enum
    impl PartialEqExceptNested for () {
        fn eq_except_nested(&self, _other: &Self) -> bool {
            true
        }
    }

    // Recursive case: general variadic tuple
    impl<Item, Rest> PartialEqExceptNested for (Item, Rest)
    where
        Item: PartialEq,
        Rest: VariadicExt + PartialEqExceptNested,
    {
        fn eq_except_nested(&self, other: &Self) -> bool {
            let (item, rest) = self;
            let (other_item, other_rest) = other;
            item == other_item && rest.eq_except_nested(other_rest)
        }
    }

    // #[test]
    // fn test_split_key_val() {
    //     type MyGht = GhtType!(u32 => u32, u32);
    //     type InputType = var_type!(u32, u32, u32);
    //     type ResultType<'a> = (var_type!(&'a u32), var_type!(&'a u32, &'a u32));
    //     let input: HashSet<InputType> = HashSet::from_iter(
    //         [
    //             (42, 314, 43770),
    //             (42, 315, 43770),
    //             (43, 10, 600),
    //             (43, 10, 60),
    //         ]
    //         .iter()
    //         .map(|&(a, b, c)| var_expr!(a, b, c)),
    //     );
    //     let htrie = MyGht::new_from(input.clone());
    //     let output: HashSet<ResultType> =
    //         htrie.recursive_iter().map(MyGht::split_key_val).collect();
    //     let correct: HashSet<ResultType> = input
    //         .iter()
    //         .map(|(a, (b, (c, ())))| ((a, ()), (b, (c, ()))))
    //         .collect();
    //     assert_eq!(output, correct);
    // }

    // #[test]
    // fn test_key_cmp() {
    //     type MyRoot = GhtType!(u16, u32 => u64);
    //     let mut trie1 = MyRoot::default();
    //     (*trie1.get_mut_trie()).insert(var_expr!(1, 2, 3));
    //     let mut trie2 = MyRoot::default();
    //     (*trie2.get_mut_trie()).insert(var_expr!(1, 2, 4));

    //     let tup1 = trie1.recursive_iter_keys().next().unwrap();
    //     let (k1, _v1) = MyRoot::split_key_val(tup1);
    //     let tup2 = trie2.recursive_iter_keys().next().unwrap();
    //     let (k2, _v2) = MyRoot::split_key_val(tup2);
    //     assert!(tup1 != tup2);
    //     assert_eq!(k1, k2);
    // }

    use variadics_macro::tuple;

    #[test]
    fn test_tuple_macro() {
        let tup = var_expr!(1, 2, 3, "four");
        let a = tuple!(tup, 4);
        assert_eq!(a, (1, 2, 3, "four"));

        let tup = var_expr!(1, 2, var_expr!(3));
        let b = tuple!(tup, 3);
        assert_eq!(b, (1, 2, (3, ())));

        type MyRoot = GhtType!(u16, u32 => u64: Row);

        let mut trie1 = MyRoot::default();
        // Can get the len, but cannot pass it into tuple! macro anyhow
        // let len = <<MyRoot as GeneralizedHashTrie>::Schema as VariadicExt>::LEN;
        // println!("schema_length: {}", len);
        trie1.insert(var_expr!(1, 2, 3));
        let t = trie1.recursive_iter().next().unwrap();
        let tup = tuple!(t, 3);
        assert_eq!(tup, (&1, &2, &3));
    }

    use std::hash::BuildHasherDefault;

    #[test]
    fn test_triangle_generic_join() {
        use fnv::FnvHasher;
        const MATCHES: u32 = 1000;
        type MyGht = GhtType!(u32 => u32: Row);

        let r_iter = (0..MATCHES)
            .map(|i| (0, i))
            .chain((1..MATCHES).map(|i| (i, 0)));

        let s_iter = (0..MATCHES)
            .map(|i| (0, i))
            .chain((1..MATCHES).map(|i| (i, 0)));

        let t_iter = (0..MATCHES)
            .map(|i| (0, i))
            .chain((1..MATCHES).map(|i| (i, 0)));

        println!("Building GHT for rx");
        io::stdout().flush().unwrap();
        let rx_ght = MyGht::new_from(r_iter.clone().map(|(x, y)| var_expr!(x, y)));
        println!("Building GHT for sb");
        io::stdout().flush().unwrap();
        let sb_ght = MyGht::new_from(s_iter.clone().map(|(y, b)| var_expr!(b, y)));
        println!("Building GHT for tx");
        io::stdout().flush().unwrap();
        let tx_ght = MyGht::new_from(t_iter.clone().map(|(z, x)| var_expr!(x, z)));
        println!("GHTs built");
        io::stdout().flush().unwrap();

        let r_x = r_iter
            .clone()
            .map(|(x, _y)| x)
            .collect::<HashSet<_, BuildHasherDefault<FnvHasher>>>();
        let t_x = s_iter
            .clone()
            .map(|(_z, x)| x)
            .collect::<HashSet<_, BuildHasherDefault<FnvHasher>>>();
        let x_inter = r_x.intersection(&t_x);
        // let len = x_inter.clone().count();
        // if len > 1 {
        //     println!("x intersection size: {:?}", len);
        // }
        // io::stdout().flush().unwrap();

        let mut output: Vec<(u32, u32, u32)> = Vec::new();
        let mut x_iters = 0usize;
        let mut y_iters = 0usize;
        let mut z_iters = 0usize;
        for a in x_inter {
            x_iters += 1;
            let r = rx_ght
                .prefix_iter(var_expr!(a))
                .map(|(_x, (y, ()))| *y)
                .collect::<HashSet<_, BuildHasherDefault<FnvHasher>>>();
            let s_y = s_iter
                .clone()
                .map(|(y, _z)| y)
                .collect::<HashSet<_, BuildHasherDefault<FnvHasher>>>();
            let y_inter = r.intersection(&s_y);
            // let len = y_inter.clone().count();
            // if len > 1 {
            //     println!("y intersection size of a = {}: {:?}", a, len);
            // }
            // io::stdout().flush().unwrap();
            for b in y_inter {
                y_iters += 1;
                let s = sb_ght
                    .prefix_iter(var_expr!(b))
                    .map(|(_b, (z, ()))| *z)
                    .collect::<HashSet<_, BuildHasherDefault<FnvHasher>>>();
                let t = tx_ght
                    .prefix_iter(var_expr!(a))
                    .map(|(_x, (z, ()))| *z)
                    .collect::<HashSet<_, BuildHasherDefault<FnvHasher>>>();
                let z_inter = s.intersection(&t);
                // let len = z_inter.clone().count();
                // if len > 1 {
                //     println!("intersection size of a = {}, b = {}: {:?}", a, b, len);
                // }
                // io::stdout().flush().unwrap();
                for c in z_inter {
                    z_iters += 1;
                    // println!("Inserting ({}, {}, {})", a, b, c);
                    output.push((*a, *b, *c));
                }
            }
        }
        // let mut output = Vec::from_iter(output.iter());
        // output.sort();
        // output
        //     .iter()
        //     .enumerate()
        //     .for_each(|(i, (a, b, c))| println!("gj #{}: ({}, {}, {})", i, a, b, c));
        println!("output size: {}", output.len());
        println!(
            "x_iters: {}, y_iters: {}, z_iters:{}",
            x_iters, y_iters, z_iters
        );
    }

    fn clover_setup(
        matches: usize,
    ) -> (
        impl Iterator<Item = (u32, u32)>,
        impl Iterator<Item = (u32, u32)>,
        impl Iterator<Item = (u32, u32)>,
    ) {
        let r_iter = (1..matches)
            .map(|i| (1u32, i as u32))
            .chain((1..matches).map(|i| (2, i as u32)))
            .chain([(0, 0)]);

        let s_iter = (1..matches)
            .map(|i| (2u32, i as u32))
            .chain((1..matches).map(|i| (3, i as u32)))
            .chain([(0, 0)]);

        let t_iter = (1..matches)
            .map(|i| (3u32, i as u32))
            .chain((1..matches).map(|i| (1, i as u32)))
            .chain([(0, 0)]);
        (r_iter, s_iter, t_iter)
    }
    #[test]
    fn clover_generic_join() {
        const MATCHES: usize = 1000;

        // let r_iter = (0..MATCHES)
        //     .map(|i| (0, i))
        //     .chain((1..MATCHES).map(|i| (i, 0)));

        let (r_iter, s_iter, t_iter) = clover_setup(MATCHES);

        type MyGht = GhtType!(u32 => u32: Row);
        let rx_ght = MyGht::new_from(r_iter.map(|(x, a)| var_expr!(x, a)));
        let sx_ght = MyGht::new_from(s_iter.map(|(x, b)| var_expr!(x, b)));
        let tx_ght = MyGht::new_from(t_iter.map(|(x, c)| var_expr!(x, c)));
        for x in rx_ght.iter() {
            if let (Some(r), Some(s), Some(t)) = (rx_ght.get(&x), sx_ght.get(&x), tx_ght.get(&x)) {
                // All unwraps succeeded, use `r`, `s`, `t` here
                for a in r.iter() {
                    for b in s.iter() {
                        for c in t.iter() {
                            println!("clover output: ({:?}, {}, {}, {})", x, a, b, c);
                            assert_eq!((x, a, b, c), (0, 0, 0, 0));
                        }
                    }
                }
            } else {
                // If any unwrap fails, continue to the next iteration
                continue;
            }
        }
    }
    #[test]
    fn clover_factorized_join() {
        const MATCHES: usize = 1000;

        // let r_iter = (0..MATCHES)
        //     .map(|i| (0, i))
        //     .chain((1..MATCHES).map(|i| (i, 0)));

        let (r_iter, s_iter, t_iter) = clover_setup(MATCHES);

        type Ght1 = GhtType!(() => u32, u32: Row);
        type Ght2 = GhtType!(u32 => u32: Row);
        let rx_ght = Ght1::new_from(r_iter.map(|(x, a)| var_expr!(x, a)));
        let sx_ght = Ght2::new_from(s_iter.map(|(x, b)| var_expr!(x, b)));
        let tx_ght = Ght2::new_from(t_iter.map(|(x, c)| var_expr!(x, c)));

        for t in rx_ght.recursive_iter() {
            let (x, (a, ())): (&u32, (&u32, _)) = t;
            if let (Some(s), Some(t)) = (sx_ght.get(x), tx_ght.get(x)) {
                // All unwraps succeeded, use `s`, `t` here
                for b in s.iter() {
                    for c in t.iter() {
                        println!("clover output: ({}, {}, {}, {})", x, a, b, c);
                        assert_eq!((x, a, b, c), (&0, &0, 0, 0));
                    }
                }
            } else {
                // If any unwrap fails, continue to the next iteration
                continue;
            }
        }
    }

    #[test]
    fn test_force() {
        type LeafType = GhtType!(() => u16, u32, u64: Row);
        let n = LeafType::new_from(vec![
            var_expr!(1, 1, 1),
            var_expr!(1, 2, 2),
            var_expr!(1, 3, 3),
            var_expr!(2, 4, 4),
        ]);
        let out = n.force().unwrap();
        // println!("resulting trie is {:?}", out);
        assert_eq!(out.height(), 1);
    }

    #[test]
    fn test_ght_forest() {
        // [[𝑅(𝑥),𝑆(𝑥),𝑇(𝑥),T'(x)], [𝑅(𝑎)], [𝑆(𝑏)], [T'(c), 𝑇(𝑐)]]
        //
        // R(a, x, y), S(a, x, c), T(a, x, c)
        // [[R(a, x),S(a, x),T(a, x)], [T(a, c), S(a, c)]]
        type LeafType = GhtType!(() => u16, u32, u64: Column);

        let table_r = LeafType::new_from(vec![
            var_expr!(0, 1, 1),
            var_expr!(0, 1, 2),
            var_expr!(0, 1, 3),
            var_expr!(1, 2, 1),
            var_expr!(1, 3, 1),
        ]);
        let table_s = LeafType::new_from(vec![
            var_expr!(0, 1, 1),
            var_expr!(0, 1, 2),
            var_expr!(0, 1, 3),
            var_expr!(0, 2, 1),
            var_expr!(0, 2, 2),
            var_expr!(0, 2, 3),
            var_expr!(1, 2, 1),
            var_expr!(1, 2, 2),
            var_expr!(1, 2, 3),
            var_expr!(1, 3, 1),
            var_expr!(1, 3, 2),
            var_expr!(1, 3, 3),
        ]);
        let table_t = LeafType::new_from(vec![
            var_expr!(0, 1, 1),
            var_expr!(0, 1, 2),
            var_expr!(0, 1, 3),
            var_expr!(1, 3, 1),
        ]);

        // set up S forest
        type SForest = GhtForestType!(u16, u32, u64);
        let mut s_forest = SForest::default();
        s_forest.0 = table_s;
        assert!(s_forest.0.height() == 0);
        assert!(s_forest.1 .0.height() == 1);
        assert!(s_forest.1 .1 .0.height() == 2);

        // set up T forest
        type TForest = GhtForestType!(u16, u32, u64);
        let mut t_forest = TForest::default();
        t_forest.0 = table_t;

        // remainder of original test
        for r in table_r.elements {
            println!("r tup is {:?}", r);
            s_forest.find_containing_leaf(r.as_ref_var());

            // walk forest and look for (r.0, r.1, *)
            // first check s_forest.0

            // fn check_prefix_iter<'a, T0, T1>(_s_forest: &'a var_type!(T0, T1))
            // where
            //     T1: HtPrefixIter<var_type!(&'a u16, &'a u32, &'a var_type!(u64))>,
            // {
            // }
            // check_prefix_iter(&s_forest);

            // if contains_key(s_forest.as_mut_var(), r, key_len)
            //     && contains_key(t_forest.as_mut_var(), r, key_len)
            // {
            //     println!("matched {:?}", r);
            // }
        }
    }

    #[test]
    fn test_forest_macro() {
        type Forest4 = GhtForestType!(u8, u16, u32, u64);
        let _f4 = Forest4::default();

        type Forest3 = GhtForestType!(u8, u16, u32);
        let _f3 = Forest3::default();

        type Forest2 = GhtForestType!(u8, u16);
        let _f2 = Forest2::default();

        type Forest1 = GhtForestType!(u8);
        let _f2 = Forest1::default();
    }

    #[test]
    fn test_colt_little_get() {
        type MyForest = GhtForestType!(u8);

        // debugging info
        // type MyForest = (GhtLeaf<(u8, ()), (u8, ()), VariadicColumnMultiset<(u8, ())>>, (GhtInner<u8, GhtLeaf<(u8, ()), (), VariadicColumnMultiset<(u8, ())>>>, ()));
        let mut forest = MyForest::default();
        // type Force = crate::ght::GhtInner<u8, GhtLeaf<(u8, ()), (), VariadicColumnMultiset<(u8, ())>>>;
        // let force: Force = forest.0.force().unwrap();


        forest.0.insert(var_expr!(1));
        forest.0.insert(var_expr!(2));
        forest.0.insert(var_expr!(3));

        println!("forest.len() = {}", forest.len());
        println!("forest before get: {:?}", forest);

        // 343  |   impl<'a, Rest, Schema, SuffixSchema, Storage> ColtNode for var_type!(&'a mut GhtLeaf<Schema, SuffixSchema, Storage>, ...Rest)
        //      |                                                 --------     ------------------------------------------------------------------
        // 344  |   where
        // 345  |       Rest: ColtNodeTail<
        //      |  ___________^
        // 346  | |         <GhtLeaf<Schema, SuffixSchema, Storage> as ColumnLazyTrieNode>::Force,
        // 347  | |         Schema = Schema,
        // 348  | |         // SuffixSchema = SuffixSchema,
        // 349  | |     >,
        //      | |_____^ unsatisfied trait bound introduced here
        let result = ColtNode::get(forest.as_mut_var(), &3);
        // println!("forest after get: {:?}", forest);
        // println!("result.len() = {}", result.len());
        // let result2 = result.get(&4);
        // println!("forest.1: {:?}", forest.1);
    }

    #[test]
    fn test_build_forest() {
        type MyForest = GhtForestType!(u8, u16, u32, u64);
        let mut forest = MyForest::default();
        forest.0.insert(var_expr!(1, 1, 1, 1));
        forest.1 .0.insert(var_expr!(2, 2, 2, 2));
        forest.1 .1 .0.insert(var_expr!(3, 3, 3, 3));

        // let i = <MyForest as GhtForest<<var_type!(u8, u16, u32, u64) as VariadicExt>::AsRefVar<'_>>>::find_matching_trie(&forest, var_expr!(1, 1, 1, 1).as_ref_var(), 0);
        assert_eq!(
            // println!(
            //     "found in trie {}",
            ForestFindLeaf::<
                var_type!(u8, u16, u32, u64),
                VariadicColumnMultiset<var_type!(u8, u16, u32, u64)>,
            >::find_containing_leaf(
                &forest, var_expr!(1_u8, 1_u16, 1_u32, 1_u64).as_ref_var()
            )
            .unwrap()
            .iter_tuples()
            .next()
            .unwrap(),
            var_expr!(1, 1, 1, 1).as_ref_var()
        );
        assert_eq!(
            // println!(
            //     "found in trie {}",
            ForestFindLeaf::<
                var_type!(u8, u16, u32, u64),
                VariadicColumnMultiset<var_type!(u8, u16, u32, u64)>,
            >::find_containing_leaf(&forest, var_expr!(2, 2, 2, 2).as_ref_var())
            .unwrap()
            .iter_tuples()
            .next()
            .unwrap(),
            var_expr!(2, 2, 2, 2).as_ref_var()
        );
        assert_eq!(
            // println!(
            //     "found in trie {}",
            ForestFindLeaf::<
                var_type!(u8, u16, u32, u64),
                VariadicColumnMultiset<var_type!(u8, u16, u32, u64)>,
            >::find_containing_leaf(&forest, var_expr!(3, 3, 3, 3).as_ref_var())
            .unwrap()
            .iter_tuples()
            .next()
            .unwrap(),
            var_expr!(3, 3, 3, 3).as_ref_var()
        );
        assert!(
            // println!(
            //     "found in trie {}",
            ForestFindLeaf::<
                var_type!(u8, u16, u32, u64),
                VariadicColumnMultiset<var_type!(u8, u16, u32, u64)>,
            >::find_containing_leaf(&forest, var_expr!(4, 4, 4, 4).as_ref_var())
            .is_none()
        );
        // println!("{:?}", forest.forest);
    }

    #[test]
    fn test_force_forest() {
        type MyForest = GhtForestType!(u8, u16, u32, u64);
        let mut forest = MyForest::default();
        forest.0.insert(var_expr!(1, 1, 1, 1));
        forest.0.insert(var_expr!(2, 2, 2, 2));
        forest.0.insert(var_expr!(3, 3, 3, 3));

        GhtForest::<var_type!(u8, u16, u32, u64)>::find_and_force(
            &mut forest,
            var_expr!(1, 1, 1, 1),
        );
        println!("Forest after forcing (1, 1, 1, 1): {:?}", forest);
        GhtForest::<var_type!(u8, u16, u32, u64)>::find_and_force(
            &mut forest,
            var_expr!(2, 1, 1, 1),
        );
        println!("Forest after forcing (2, 1, 1, 1): {:?}", forest);
        GhtForest::<var_type!(u8, u16, u32, u64)>::find_and_force(
            &mut forest,
            var_expr!(3, 3, 3, 3),
        );
        println!("Forest after forcing (3, 3, 3, 3): {:?}", forest);

        println!(
            "(1, 1, 1, 1) leaf: {:?}",
            forest.find_containing_leaf(var_expr!(1, 1, 1, 1).as_ref_var())
        );
        println!(
            "(2, 1, 1, 1) leaf: {:?}",
            forest.find_containing_leaf(var_expr!(2, 1, 1, 1).as_ref_var())
        );
        println!(
            "(3, 3, 3, 3) leaf: {:?}",
            forest.find_containing_leaf(var_expr!(3, 3, 3, 3).as_ref_var())
        );
    }

    #[test]
    fn test_scale_forest() {
        type MyForest = GhtForestType!(bool, usize, &'static str => i32);
        let mut forest = MyForest::default();

        forest.0.insert(var_expr!(true, 1, "hello", -5));
        assert_eq!(forest.0.recursive_iter().count(), 1);
        for i in 1..1000000 {
            forest.0.insert(var_expr!(true, 1, "hello", i));
        }
        for i in 1..10 {
            forest.0.insert(var_expr!(true, 2, "hello", i));
        }
        assert_eq!(forest.0.recursive_iter().count(), 1000009);
        let leaf = ForestFindLeaf::<
            var_type!(bool, usize, &'static str, i32),
            VariadicColumnMultiset<var_type!(bool, usize, &'static str, i32)>,
        >::find_containing_leaf(
            &forest, var_expr!(true, 2, "hello", 2).as_ref_var()
        );
        println!("leaf size: {}", leaf.unwrap().elements.len());
        // println!("forest.0: {:?}", forest.0);
        println!("forest.1: {:?}", forest.1 .0);
    }

    #[test]
    fn test_colt_get() {
        type MyForest = GhtForestType!(u8, u16, u32, u64);
        let mut forest = MyForest::default();
        forest.0.insert(var_expr!(1, 1, 1, 1));
        forest.0.insert(var_expr!(2, 2, 2, 2));
        forest.0.insert(var_expr!(3, 3, 3, 3));

        // GhtForest::<var_type!(u8, u16, u32, u64)>::force(&mut forest, var_expr!(1, 1, 1, 1));
        let len = forest.len();
        // println!("Forest after forcing (1, 1, 1, 1): {:?}", forest);
        {
            let get_result = ColtNode::get(forest.as_mut_var(), &1);
            assert_eq!(get_result.len(), len - 1);
            assert_eq!(get_result.0.height(), 0);
            let get_result2 = ColtNode::get(get_result, &1);
            assert_eq!(get_result2.len(), len - 2);
            // assert!(get_result2.0.is_none());
            let get_result3 = ColtNode::get(get_result2, &1);
            // assert!(get_result3.1 .0.is_none());
            assert_eq!(get_result3.len(), len - 3);
            println!("initial result: {:?}", get_result3);
        }
        {
            let get_result = ColtNode::get(forest.as_mut_var(), &3);
            assert_eq!(get_result.len(), len - 1);
            let get_result2 = ColtNode::get(get_result, &3);
            println!("secondary result: {:?}", get_result2);
        }
        println!("final forest: {:#?}", forest);
    }

    #[test]
    fn test_colt_scale() {
        type MyColt = crate::GhtForestType!(i32, bool, usize, &'static str);
        let mut forest = MyColt::default();
        for i in 1..100000 {
            forest.0.insert(var_expr!(i, true, 1, "hello"));
        }
        {
            let result = forest.as_mut_var().get(&3);
            println!("result: {:?}", result);
        }
        // check: first Leaf trie is forced
        assert!(forest.0.forced);
        assert_eq!(forest.0.elements.len(), 0);
        {
            let result = forest.as_mut_var().get(&3);
            let result2 = result.get(&true);
            println!("result2: {:?}", result2);
        }
        {
            // check: leaf below 3 in first non-empty trie is forced
            let result = forest.as_mut_var().get(&3);
            assert!(result.0.forced);
            assert_eq!(result.0.elements.len(), 0);
        }
        // check: prefix (3, true) is now found in the third trie: forest.1.1.0
        assert!(forest
            .1
             .1
             .0
            .prefix_iter(var_expr!(3, true).as_ref_var())
            .next()
            .is_some());
        println!("forest.0: {:?}", forest.0);
        {
            let result = forest.as_mut_var().get(&3);
            let result2 = result.get(&true);
            let result3 = result2.get(&1);
            let result4 = result3.get(&"hello");
            println!("result4: {:?}", result4);
        }
    }
}
