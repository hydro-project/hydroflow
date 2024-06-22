#[cfg(test)]
mod tests {
    use variadics::{var_expr, VariadicExt};

    use crate::ght::{GeneralizedHashTrie, GeneralizedHashTrieNode};
    use crate::ght_lattice::{
        DeepJoinLatticeBimorphism, GhtBimorphism, GhtCartesianProductBimorphism,
        GhtNodeKeyedBimorphism,
    };
    use crate::{GhtNodeType, GhtType, LatticeBimorphism, Merge, NaiveLatticeOrd};

    #[test]
    fn basic_test() {
        // Example usage
        type MyTrie1 = GhtType!(u32, u32 => &'static str);
        let htrie1 = MyTrie1::new_from(vec![var_expr!(42, 314, "hello")]);
        assert!(htrie1.contains(var_expr!(&42, &314, &"hello")));
        assert_eq!(htrie1.recursive_iter().count(), 1);

        type MyTrie2 = GhtType!(u32 => u32);
        let htrie2 = MyTrie2::new_from(vec![var_expr!(42, 314)]);
        assert!(htrie2.contains(var_expr!(&42, &314)));
        assert_eq!(htrie1.recursive_iter().count(), 1);
    }

    #[test]
    fn test_ght_node_type_macro() {
        type LilTrie = GhtNodeType!(() => u32);
        let _l = LilTrie::new_from(vec![var_expr!(1)]);

        type SmallTrie = GhtNodeType!(u32 => &'static str);
        type SmallKeyedTrie = GhtNodeType!(u32 => &'static str);
        let l = SmallTrie::new_from(vec![var_expr!(1, "hello")]);
        let _: SmallKeyedTrie = l;

        type LongKeySmallValTrie = GhtNodeType!(u32, u16 => &'static str);
        type LongKeySmallValKeyedTrie = GhtNodeType!(u32, u16 => &'static str);
        let x = LongKeySmallValTrie::new_from(vec![var_expr!(1, 314, "hello")]);
        let _: LongKeySmallValKeyedTrie = x;
        let _ = LongKeySmallValTrie::new_from(vec![var_expr!(1, 314, "hello")]);

        type SmallKeyLongValTrie = GhtNodeType!(u32 => u64, u16, &'static str);
        let _x = SmallKeyLongValTrie::new_from(vec![var_expr!(1, 999, 222, "hello")]);

        type LongKeyLongValTrie = GhtNodeType!(u32, u64 => u16, &'static str);
        let _x = LongKeyLongValTrie::new_from(vec![var_expr!(1, 999, 222, "hello")]);
    }

    #[test]
    fn test_insert() {
        let mut htrie = <GhtType!(u16, u32 => u64)>::default();
        htrie.insert(var_expr!(42, 314, 43770));
        assert_eq!(htrie.recursive_iter().count(), 1);
        htrie.insert(var_expr!(42, 315, 43770));
        assert_eq!(htrie.recursive_iter().count(), 2);
        htrie.insert(var_expr!(42, 314, 30619));
        assert_eq!(htrie.recursive_iter().count(), 3);
        htrie.insert(var_expr!(43, 10, 600));
        assert_eq!(htrie.recursive_iter().count(), 4);
        assert!(htrie.contains(var_expr!(&42, &314, &30619)));
        assert!(htrie.contains(var_expr!(&42, &315, &43770)));
        assert!(htrie.contains(var_expr!(&43, &10, &600)));

        type LongKeyLongValTrie = GhtType!(u32, u64 => u16, &'static str);
        let mut htrie = LongKeyLongValTrie::new_from(vec![var_expr!(1, 999, 222, "hello")]);
        htrie.insert(var_expr!(1, 999, 111, "bye"));
        htrie.insert(var_expr!(1, 1000, 123, "cya"));
        println!("htrie: {:?}", htrie);
        assert!(htrie.contains(var_expr!(&1, &999, &222, &"hello")));
        assert!(htrie.contains(var_expr!(&1, &999, &111, &"bye")));
        assert!(htrie.contains(var_expr!(&1, &1000, &123, &"cya")));
    }

    #[test]
    fn test_scale() {
        type MyGht = GhtNodeType!(bool, usize, &'static str => i32);
        let mut htrie = MyGht::new_from(vec![var_expr!(true, 1, "hello", -5)]);
        assert_eq!(htrie.recursive_iter().count(), 1);
        for i in 1..1000000 {
            htrie.insert(var_expr!(true, 1, "hello", i));
        }
        assert_eq!(htrie.recursive_iter().count(), 1000000);
    }

    #[test]
    fn test_contains() {
        type MyGht = GhtType!(u16, u32 => u64);
        let htrie = MyGht::new_from(vec![var_expr!(42_u16, 314_u32, 43770_u64)]);
        println!("HTrie: {:?}", htrie);
        let x = var_expr!(&42, &314, &43770);
        assert!(htrie.contains(x));
        assert!(htrie.contains(var_expr!(42, 314, 43770).as_ref_var()));
        assert!(htrie.contains(var_expr!(&42, &314, &43770)));
        assert!(!htrie.contains(var_expr!(42, 314, 30619).as_ref_var()));
        assert!(!htrie.contains(var_expr!(&42, &315, &43770)));
        assert!(!htrie.contains(var_expr!(&43, &314, &43770)));
    }

    #[test]
    fn test_recursive_iter() {
        type MyGht = GhtType!(u32, u32 => u32);
        let mut htrie = MyGht::new_from(vec![var_expr!(42, 314, 43770)]);
        htrie.insert(var_expr!(42, 315, 43770));
        htrie.insert(var_expr!(42, 314, 30619));
        htrie.insert(var_expr!(43, 10, 600));
        for row in htrie.recursive_iter() {
            println!("{:?}", row);
        }
    }

    #[test]
    fn test_get() {
        type MyGht = GhtType!(u32, u32 => u32);
        let ht_root = MyGht::new_from(vec![var_expr!(42, 314, 43770)]);

        let inner = ht_root.get_trie().get(&42).unwrap();
        let t = inner.recursive_iter().next().unwrap();
        assert_eq!(t, var_expr!(&314, &43770));

        let leaf = inner.get(&314).unwrap();
        let t = leaf.recursive_iter().next().unwrap();
        assert_eq!(t, var_expr!(&43770));
    }

    #[test]
    fn test_iter() {
        type MyGht = GhtType!(u32, u32 => u32);
        let ht_root = MyGht::new_from(vec![var_expr!(42, 314, 43770)]);
        let inner_key = ht_root.get_trie().iter().next().unwrap();
        let inner = ht_root.get_trie().get(inner_key).unwrap();
        let t = inner.recursive_iter().next().unwrap();
        assert_eq!(t, var_expr!(&314, &43770));

        let leaf_key = inner.iter().next().unwrap();
        let leaf = inner.get(leaf_key).unwrap();
        let t = leaf.iter().next().unwrap();
        assert_eq!(t, &var_expr!(43770));
    }

    #[test]
    fn test_prefix_iter() {
        type MyGht = GhtType!(u32, u32 => u32);
        let mut htrie = MyGht::new_from(vec![var_expr!(42, 314, 43770)]);

        htrie.insert(var_expr!(42, 315, 43770));
        htrie.insert(var_expr!(42, 314, 30619));
        htrie.insert(var_expr!(43, 10, 600));

        for row in htrie.prefix_iter(var_expr!(42).as_ref_var()) {
            println!("42: {:?}", row);
        }

        for row in htrie.prefix_iter(var_expr!(42, 315).as_ref_var()) {
            println!("42,315: {:?}", row);
        }

        assert!(htrie
            .prefix_iter(var_expr!(42, 315).as_ref_var())
            .any(|_| true));
        // // Too long:
        // for row in htrie.prefix_iter(var_expr!(42, 315, 43770).as_ref_var()) {
        //     println!("42,315,43770: {:?}", row);
        // }

        for row in htrie.recursive_iter() {
            println!("All: {:?}", row);
        }
    }

    #[test]
    fn test_prefix_iter_complex() {
        type MyGht = GhtType!(bool, u32, &'static str => i32);
        let mut htrie = MyGht::new_from(vec![var_expr!(true, 1, "hello", -5)]);

        htrie.insert(var_expr!(true, 2, "hello", 1));
        htrie.insert(var_expr!(true, 1, "hi", -2));
        htrie.insert(var_expr!(true, 1, "hi", -3));
        htrie.insert(var_expr!(true, 1, "hi", -4));
        htrie.insert(var_expr!(true, 1, "hi", -5));
        htrie.insert(var_expr!(false, 10, "bye", 5));

        for row in htrie.prefix_iter(var_expr!(true).as_ref_var()) {
            println!("A {:?}", row);
        }
        for row in htrie.prefix_iter(var_expr!(true, 1, "hi").as_ref_var()) {
            println!("B {:?}", row);
        }
    }
    #[test]
    fn test_merge() {
        type MyGht = GhtType!(u32, u64 => u16, &'static str);

        let mut test_ght1 = MyGht::new_from(vec![var_expr!(42, 314, 10, "hello")]);
        let mut test_ght2 = MyGht::new_from(vec![var_expr!(42, 314, 10, "hello")]);

        // no change on merge
        assert!(!test_ght1.merge(test_ght2.clone()));

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
        type MyGht = GhtType!(u32, u64 => u16, &'static str);
        type MyGhtNode = GhtNodeType!(u32, u64 => u16, &'static str);

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

        for ght in test_vec_wrap.iter().map(|x| x.get_trie().clone()) {
            ght.naive_cmp(&ght.clone());
            test_vec.push(ght);
        }
        crate::test::check_all(&test_vec);
        crate::test::check_all(&test_vec_wrap);
    }

    #[test]
    fn test_bimorphism() {
        type MyGhtA = GhtType!(u32, u64 => u16, &'static str);
        type MyGhtB = GhtType!(u32, u64, u16 => &'static str);

        let mut ght_a = MyGhtA::default();
        let mut ght_b = MyGhtB::default();

        ght_a.insert(var_expr!(123, 2, 5, "hello"));
        ght_a.insert(var_expr!(50, 1, 1, "hi"));
        ght_a.insert(var_expr!(5, 1, 7, "hi"));
        ght_b.insert(var_expr!(5, 1, 8, "hi"));
        ght_b.insert(var_expr!(10, 1, 2, "hi"));
        ght_b.insert(var_expr!(12, 10, 98, "bye"));

        type MyGhtAB = GhtNodeType!(u32, u64, u16, &'static str, u32, u64 => u16, &'static str);

        let mut bim = GhtCartesianProductBimorphism::<MyGhtAB>::default();
        let ght_out = bim.call(ght_a.get_trie(), ght_b.get_trie());
        println!(
            "{:?}: count {:?}",
            ght_out,
            ght_out.recursive_iter().count()
        );
    }

    #[test]
    fn test_join_bimorphism() {
        type MyGhtATrie = GhtNodeType!(u32, u64, u16 => &'static str);
        type MyGhtBTrie = GhtNodeType!(u32, u64, u16 => &'static str);

        let mut ght_a = MyGhtATrie::default();
        let mut ght_b = MyGhtBTrie::default();

        ght_a.insert(var_expr!(123, 2, 5, "hello"));
        ght_a.insert(var_expr!(50, 1, 1, "hi"));
        ght_a.insert(var_expr!(5, 1, 7, "hi"));

        ght_b.insert(var_expr!(5, 1, 8, "hi"));
        ght_b.insert(var_expr!(5, 1, 7, "world"));
        ght_b.insert(var_expr!(10, 1, 2, "hi"));
        ght_b.insert(var_expr!(12, 10, 98, "bye"));

        {
            type CartProd = GhtNodeType!(u64, u16, &'static str, u64, u16 => &'static str);
            let mut bim =
                GhtNodeKeyedBimorphism::new(GhtCartesianProductBimorphism::<CartProd>::default());
            let out = bim.call(&ght_a, &ght_b);
            println!("{:?}", out);
            for row in out.recursive_iter() {
                println!("ROW {:?}", row);
            }
        }

        {
            // type MyGhtOut = Vec<var_type!(u16, &'static str, u16, &'static str)>;
            type CartProd = GhtNodeType!(u16, &'static str, u16 => &'static str);
            let mut bim = GhtNodeKeyedBimorphism::new(GhtNodeKeyedBimorphism::new(
                GhtCartesianProductBimorphism::<CartProd>::default(),
            ));
            let out = bim.call(&ght_a, &ght_b);
            println!("{:?}", out);
            for row in out.recursive_iter() {
                println!("ROW {:?}", row);
            }
        }

        {
            type MyGhtOut = GhtNodeType!(&'static str => &'static str);
            let mut bim = GhtNodeKeyedBimorphism::new(GhtNodeKeyedBimorphism::new(
                GhtNodeKeyedBimorphism::new(GhtCartesianProductBimorphism::<MyGhtOut>::default()),
            ));
            let out = bim.call(&ght_a, &ght_b);
            println!("{:?}", out);
            for row in out.recursive_iter() {
                println!("ROW {:?}", row);
            }
        }
        {
            // This is a more compact representation of the block above.
            type MyNodeBim =
                <(MyGhtATrie, MyGhtBTrie) as DeepJoinLatticeBimorphism>::DeepJoinLatticeBimorphism;
            let mut bim = <MyNodeBim as Default>::default();
            let out = bim.call(&ght_a, &ght_b);
            println!("{:?}", out);
            for row in out.recursive_iter() {
                println!("ROW {:?}", row);
            }
        }
        {
            // This is an even more compact representation of the block above.
            type MyNodeBim = <MyGhtATrie as GeneralizedHashTrieNode>::DeepJoin<MyGhtBTrie>;
            let mut bim = <MyNodeBim as Default>::default();
            let out = bim.call(&ght_a, &ght_b);
            println!("{:?}", out);
            for row in out.recursive_iter() {
                println!("ROW {:?}", row);
            }
        }
    }

    #[test]
    fn test_join_bimorphism_wrapper() {
        type MyGhtA = GhtType!(u32, u64, u16 => &'static str);
        type MyGhtB = GhtType!(u32, u64, u16 => &'static str);
        type MyGhtATrie = <MyGhtA as GeneralizedHashTrie>::Trie;
        type MyGhtBTrie = <MyGhtB as GeneralizedHashTrie>::Trie;

        let mut ght_a = MyGhtA::default();
        let mut ght_b = MyGhtB::default();

        ght_a.insert(var_expr!(123, 2, 5, "hello"));
        ght_a.insert(var_expr!(50, 1, 1, "hi"));
        ght_a.insert(var_expr!(5, 1, 7, "hi"));

        ght_b.insert(var_expr!(5, 1, 8, "hi"));
        ght_b.insert(var_expr!(5, 1, 7, "world"));
        ght_b.insert(var_expr!(10, 1, 2, "hi"));
        ght_b.insert(var_expr!(12, 10, 98, "bye"));

        {
            type CartProd = GhtNodeType!(u64, u16, &'static str, u64, u16 => &'static str);
            let mut bim = GhtBimorphism::new(GhtNodeKeyedBimorphism::new(
                GhtCartesianProductBimorphism::<CartProd>::default(),
            ));
            let out = bim.call(ght_a.clone(), ght_b.clone());
            println!("{:?}", out);
            for row in out.recursive_iter() {
                println!("ROW {:?}", row);
            }
        }

        {
            // type MyGhtOut = Vec<var_type!(u16, &'static str, u16, &'static str)>;
            type CartProd = GhtNodeType!(u16, &'static str, u16 => &'static str);
            let mut bim = GhtBimorphism::new(GhtNodeKeyedBimorphism::new(
                GhtNodeKeyedBimorphism::new(GhtCartesianProductBimorphism::<CartProd>::default()),
            ));
            let out = bim.call(ght_a.clone(), ght_b.clone());
            println!("{:?}", out);
            for row in out.recursive_iter() {
                println!("ROW {:?}", row);
            }
        }

        {
            type MyGhtOut = GhtNodeType!(&'static str => &'static str);
            let mut bim = GhtBimorphism::new(GhtNodeKeyedBimorphism::new(
                GhtNodeKeyedBimorphism::new(GhtNodeKeyedBimorphism::new(
                    GhtCartesianProductBimorphism::<MyGhtOut>::default(),
                )),
            ));
            let out = bim.call(ght_a.clone(), ght_b.clone());
            println!("{:?}", out);
            for row in out.recursive_iter() {
                println!("ROW {:?}", row);
            }
        }
        {
            // This is a more compact representation of the block above.
            type MyNodeBim =
                <(MyGhtATrie, MyGhtBTrie) as DeepJoinLatticeBimorphism>::DeepJoinLatticeBimorphism;
            type MyBim = GhtBimorphism<MyNodeBim>;
            let mut bim = <MyBim as Default>::default();
            let out = bim.call(ght_a.clone(), ght_b.clone());
            println!("{:?}", out);
            for row in out.recursive_iter() {
                println!("ROW {:?}", row);
            }
        }
        {
            // This is an even more compact representation of the block above.
            type MyNodeBim = <MyGhtATrie as GeneralizedHashTrieNode>::DeepJoin<MyGhtBTrie>;
            type MyBim = GhtBimorphism<MyNodeBim>;
            let mut bim = <MyBim as Default>::default();
            let out = bim.call(ght_a.clone(), ght_b.clone());
            println!("{:?}", out);
            for row in out.recursive_iter() {
                println!("ROW {:?}", row);
            }
        }
    }

    #[test]
    fn test_recursive_iter_keys() {
        type MyGht = GhtType!(u32 => u32, u32);
        let mut htrie = MyGht::new_from(vec![var_expr!(42, 314, 43770)]);
        htrie.insert(var_expr!(42, 315, 43770));
        htrie.insert(var_expr!(43, 10, 600));
        htrie.insert(var_expr!(43, 10, 60));
        for row in htrie.recursive_iter_keys() {
            println!("row: {:?}, height: {}", row, htrie.height().unwrap());
        }
    }

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

    #[test]
    fn test_split_key_val() {
        type MyGht = GhtType!(u32 => u32, u32);
        let mut htrie = MyGht::new_from(vec![var_expr!(42, 314, 43770)]);
        htrie.insert(var_expr!(42, 315, 43770));
        htrie.insert(var_expr!(43, 10, 600));
        htrie.insert(var_expr!(43, 10, 60));
        let tup = htrie.recursive_iter().next().unwrap();
        let (k, v) = MyGht::split_key_val(tup);
        println!("key: {:?}, val {:?}", k, v);
    }

    #[test]
    fn test_key_cmp() {
        type MyRoot = GhtType!(u16, u32 => u64);
        let mut trie1 = MyRoot::default();
        (*trie1.get_mut_trie()).insert(var_expr!(1, 2, 3));
        let mut trie2 = MyRoot::default();
        (*trie2.get_mut_trie()).insert(var_expr!(1, 2, 4));

        let tup1 = trie1.get_trie().recursive_iter_keys().next().unwrap();
        let (k1, _v1) = MyRoot::split_key_val(tup1);
        let tup2 = trie2.get_trie().recursive_iter_keys().next().unwrap();
        let (k2, _v2) = MyRoot::split_key_val(tup2);
        assert!(tup1 != tup2);
        assert_eq!(k1, k2);
    }

    use variadics::var_len;
    use variadics_macro::tuple;

    #[test]
    fn test_tuple_macro() {
        let tup = var_expr!(1, 2, 3, "four");
        let a = tuple!(tup, 4);
        assert_eq!(a, (1, 2, 3, "four"));
        println!("a: {:?}", a);

        let tup = var_expr!(1, 2, var_expr!(3));
        let b = tuple!(tup, 3);
        assert_eq!(b, (1, 2, (3, ())));

        type MyRoot = GhtType!(u16, u32 => u64);

        let mut trie1 = MyRoot::default();
        let len = <<MyRoot as GeneralizedHashTrie>::Schema as VariadicExt>::LEN;
        println!("schema_length: {}", len);
        (*trie1.get_mut_trie()).insert(var_expr!(1, 2, 3));
        let t = trie1.recursive_iter().next().unwrap();
        let tup = tuple!(t, 3);
        println!("{:?}", tup);
    }

    #[test]
    fn test_var_len() {
        let i = var_len!((1, (2, (3, ()))));
        println!("{}", i);
    }
}
