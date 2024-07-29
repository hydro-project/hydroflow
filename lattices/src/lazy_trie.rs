use std::hash::Hash;

use sealed::sealed;
use variadics::{var_args, var_expr, var_type, MutVariadic, RefVariadic, Split, VariadicExt};

use crate::ght::{GeneralizedHashTrieNode, GhtHasChildren, GhtInner, GhtLeaf};
use crate::{GhtNodeType, Merge};

/// COLT from Wang/Willsey/Suciu
pub trait ColumnLazyTrieNode: GeneralizedHashTrieNode {
    /// into_iter for leaf elements, needed by force below
    fn into_iter(self) -> Option<impl Iterator<Item = Self::Schema>>;

    /// pull all the data out of this trie node but retain the reference
    fn drain(&mut self) -> Option<impl Iterator<Item = Self::Schema>>;

    /// result of `force`ing a node
    type Force: GeneralizedHashTrieNode + GhtHasChildren;
    // + for<'a> HtPrefixIter<<Self::Schema as VariadicExt>::AsRefVar<'a>>;
    /// Force the generation of a parent node, as in the Wang/Willsey/Suciu COLT structure
    fn force(self) -> Option<Self::Force>;

    /// Force the generation of a parent node but retain ref to this node
    fn force_drain(&mut self) -> Option<Self::Force>;
}

// pub trait DeepForce<Key: VariadicExt>: ColumnLazyTrieNode {
//     /// Return type of [`Self::deep_force`].
//     type DeepForce: GeneralizedHashTrieNode + GhtHasChildren;
//     /// Recurse until a leaf, force the leaf and build superstructure 1 level taller
//     fn deep_force(&mut self, key: Key) -> Self::DeepForce;
// }

impl<Head, Node> ColumnLazyTrieNode for GhtInner<Head, Node>
where
    Head: 'static + Hash + Eq,
    Node: 'static + ColumnLazyTrieNode,
{
    fn into_iter(self) -> Option<impl Iterator<Item = var_type!(Head, ...Node::Schema)>> {
        None::<Box<dyn Iterator<Item = Self::Schema>>>
    }

    fn drain(&mut self) -> Option<impl Iterator<Item = Self::Schema>> {
        None::<Box<dyn Iterator<Item = Self::Schema>>>
    }

    type Force = GhtInner<Head, Node>;
    fn force(self) -> Option<Self::Force> {
        None
    }

    fn force_drain(&mut self) -> Option<GhtInner<Head, Node>> {
        None
    }
}

// impl<Head, Node> DeepForce for GhtInner<Head, Node>
// where
//     Head: 'static + Hash + Eq,
//     Node: 'static + DeepForce,
// {
//     type DeepForce = GhtInner<Head, Node::DeepForce>;
//     fn deep_force(&mut self) -> Self::DeepForce {
//         todo!()
//     }
// }

impl<Head, Rest> ColumnLazyTrieNode for GhtLeaf<var_type!(Head, ...Rest)>
where
    Head: 'static + Hash + Eq,
    Rest: 'static + Hash + Eq + VariadicExt,
    // for<'r, 's> <var_type!(Head, ...Rest) as VariadicExt>::AsRefVar<'r>:
    //     PartialEq<<var_type!(Head, ...Rest) as VariadicExt>::AsRefVar<'s>>,
    for<'r> Rest::AsRefVar<'r>: PartialEq<Rest::AsRefVar<'r>>,
{
    fn into_iter(self) -> Option<impl Iterator<Item = Self::Schema>> {
        Some(self.elements.into_iter())
    }

    fn drain(&mut self) -> Option<impl Iterator<Item = Self::Schema>> {
        Some(self.elements.drain())
    }

    type Force = GhtInner<Head, GhtLeaf<Rest>>;
    fn force(self) -> Option<Self::Force> {
        let mut retval = Self::Force::default();
        for t in self.into_iter().unwrap() {
            let var_expr!(h, ...r) = t;
            retval.insert((h, r));
        }
        Some(retval)
    }

    fn force_drain(&mut self) -> Option<GhtInner<Head, GhtLeaf<Rest>>> {
        let mut retval = Self::Force::default();
        for t in self.drain().unwrap() {
            let var_expr!(h, ...r) = t;
            retval.insert((h, r));
        }
        Some(retval)
    }
}

#[macro_export]
macro_rules! GhtForestType {
    ($a:tt, $( $b:tt ),* => ()) => {
        ()
    };
    ($a:tt => $c:tt, $( $d:tt ),* ) => {
        (GhtNodeType!($a => $c, $( $d ),*), GhtForestType!($a, $c => $( $d ),*))
    };
    ($a:tt, $( $b:tt ),* => $c:tt) => {
        (GhtNodeType!($a, $( $b ),* => $c), ())
    };

    ($a:tt, $( $b:tt ),* => $c:tt, $( $d:tt ),* ) => {
        (GhtNodeType!($a, $( $b ),* => $c, $( $d ),*), GhtForestType!($a, $( $b ),* , $c => $( $d ),*))
    };

    ($a:tt, $( $b:tt ),* ) => {
        GhtForestType!($a => $( $b ),*)
    };

    ($head:tt, ($rest:tt)) => {
        GhtForestType!($head => $rest )
    };
}

// #[sealed]
// pub trait ColtForest<'a, Prefix> {
//     /// type of the suffix of this prefix
//     type Suffix: VariadicExt;

//     /// Forest variadic: a list of lazy tries of increasing height over the same schema.
//     /// The union of these tries is equivalent to a Willsey/Wang/Suciu Column-Oriented Lazy Trie (COLT).
//     /// However, instead of having a trie with paths of different lengths that split at leaves, we
//     /// have a forest of tries, each one of uniform height (all paths the same length).
//     /// The first trie is height k, the second height k+1, ... and the last is height schema-length.
//     /// As we move right in the forest, the leaves have 1 less column on the left, and one extra parent
//     /// layer holding an HtInner for that column
//     /// For example, in schema like var_type!(u8, u16, u32, u64)
//     ///    - the first trie in the forest is HtInner<u8, HtLeaf<var_type!(u16, u32, u64)>>
//     ///    - the second trie in the forest is HtInner<u8, HtInner<u16, HtLeaf<var_type!(u32, u64)>>>
//     ///    - the third trie in the forest is HtInner<u8, HtInner<u16, HtInner<u32, HtLeaf<var_type!(u64)>>>>
//     /// When we need to `force` a leaf in one trie, we don't split that leaf as they do in COLT;
//     /// instead we remove it from that trie and merge it into the next trie to the right.
//     // type Forest: VariadicExt;

//     /// given a prefix, look through tries right-to-left until we find a match (or fail)
//     fn get_leaf(
//         self,
//         prefix: Prefix,
//     ) -> Option<&'a GhtLeaf<<Self::Suffix as VariadicExt>::AsRefVar<'a>>>
//     where
//         Self::Suffix: 'a,
//         // <<Self as ColtForest<Prefix>>::Suffix as VariadicExt>::AsRefVar<'a>: Eq + Hash,
//         <Self::Suffix as VariadicExt>::AsRefVar<'a>: Eq + Hash;
// }

// // #[derive(Debug, Default)]
// // pub struct ColtForestStruct<T> {
// //     pub forest: T, // T should be formed via GhtForestType!
// // }

// // // search_key is always shorter than the schema
// // /// Does this GHT Forest contain matches to this search key?
// // example schema: var!(c1, c2, c3, c4, c5)
// // search_key: var!(Sk1, Sk2, Sk3)
// // forest: var!(Trie1, Trie2, Trie3, Trie4)
// //    - Trait 1: search_key length > forest height, must force
// //        - Trie1(c1 => c2, c3, c4, c5): if matches c1, force leaf
// //             if force: force the matching leaf, give it a new parent with `c1`, and merge right into `Trie2`
// //        - Trie2(c1, c2 => c3, c4, c5): if matches c1, recurse. Then if matches c2, force leaf
// //             if find matching leaf: force the leaf, give it new parent with c2, new grandparent with c1, and merge right into `Trie3`
// //    - Trait 2: search_key length <= forest height, no force
// //        - Trie3(c1, c2, c3 => c4, c5): no force
// //        - Trie4(c1, c2, c3, c4 => c5): no force

// // impl<Key> ColtForest<Key> for ColtForest<()> {
// //     type Suffix = ;
// // }
// #[sealed]
// impl<'k, ThisKey, RestOfKeys, ThisTrie> ColtForest<'k, var_type!(&'k ThisKey, ...RestOfKeys)>
//     for var_type!(&'k mut ThisTrie)
// where
//     ThisKey: Eq + Hash,
//     RestOfKeys: VariadicExt + Eq + Hash + RefVariadic,
//     ThisTrie: GeneralizedHashTrieNode<Head = ThisKey> + GhtHasChildren + ColumnLazyTrieNode,
//     ThisTrie::Schema: 'k + Split<var_type!(ThisKey,  ...RestOfKeys::UnRefVar)>,
//     <ThisTrie::Schema as Split<var_type!(ThisKey, ...RestOfKeys::UnRefVar)>>::Suffix:
//         'k + VariadicExt,
//     <<ThisTrie::Schema as Split<(ThisKey, RestOfKeys::UnRefVar)>>::Suffix as VariadicExt>::AsRefVar<
//         'k,
//     >: 'k + Eq + Hash,
// {
//     type Suffix = <<ThisTrie as GeneralizedHashTrieNode>::Schema as Split<
//         var_type!(ThisKey,  ...RestOfKeys::UnRefVar),
//     >>::Suffix;

//     fn get_leaf(
//         self,
//         prefix: var_type!(&'k ThisKey, ...RestOfKeys),
//     ) -> Option<&GhtLeaf<<Self::Suffix as VariadicExt>::AsRefVar<'k>>> {
//         drop(prefix);
//         todo!()
//     }
// }

// #[sealed]
// impl<'k, ThisKey, NextKey, RestOfKeys, ThisTrie, NextTrie, RestOfTries>
//     ColtForest<'k, var_type!(&'k ThisKey, &'k NextKey, ...RestOfKeys)> for var_type!(&'k mut ThisTrie, &'k mut NextTrie, ...RestOfTries)
// where
//     ThisKey: Eq + Hash + std::fmt::Debug,
//     NextKey: Eq + Hash,
//     RestOfKeys: VariadicExt + Eq + Hash + RefVariadic,
//     ThisTrie: GeneralizedHashTrieNode<Head = ThisKey> + GhtHasChildren + ColumnLazyTrieNode,
//     NextTrie: GeneralizedHashTrieNode<Head = NextKey>
//         + GhtHasChildren
//         + ColumnLazyTrieNode
//         + Merge<NextTrie>,
//     RestOfTries: VariadicExt + MutVariadic,
//     ThisTrie::Node:
//         GeneralizedHashTrieNode + ColumnLazyTrieNode<Force = <NextTrie as GhtHasChildren>::Node>,
//     ThisTrie::Schema: Split<var_type!(ThisKey, NextKey, ...RestOfKeys::UnRefVar)>,
//     <ThisTrie::Schema as Split<var_type!(ThisKey, NextKey, ...RestOfKeys::UnRefVar)>>::Suffix:
//         VariadicExt,
//     // var_type!(&'k mut NextTrie, ...RestOfTries):
//     //     ColtForest<'k, var_type!(ThisKey, NextKey, ...RestOfKeys)>,
// {
//     type Suffix = <<ThisTrie as GeneralizedHashTrieNode>::Schema as Split<
//         var_type!(ThisKey, NextKey, ...RestOfKeys::UnRefVar),
//     >>::Suffix; // var_type!(NextKey, ...RestOfKeys);

//     fn get_leaf(
//         self,
//         prefix: var_type!(&'k ThisKey, &'k NextKey, ...RestOfKeys),
//     ) -> Option<&GhtLeaf<<Self::Suffix as VariadicExt>::AsRefVar<'k>>>
//     where
//         Self::Suffix: 'k,
//         <Self::Suffix as VariadicExt>::AsRefVar<'k>: Eq + Hash,
//         ThisKey: std::fmt::Debug,
//     {
//         let var_args!(sk_first, sk_next, ...sk_rest) = prefix;
//         let var_args!(trie_first, trie_next, ...trie_rest) = self;
//         println!("handling key {:?}", sk_first);
//         // if no force needed we can traverse to leaf here
//         //    if found we return it.
//         //    if not found we drop through and move to next trie in the forest
//         // else traverse to leaf and force
//         //     force detaches the leaf from trie_first and merges it into trie_next

//         // now move to next trie in forest
//         let sub_forest = var_expr!(trie_next, ...trie_rest);
//         // fn check<T>(x: impl ColtForest<T>) {}
//         // check(sub_forest);
//         <var_type!(&mut NextTrie, ...RestOfTries) as ColtForest<
//             var_type!(ThisKey, NextKey, ...RestOfKeys),
//         >>::get_leaf(sub_forest, prefix)
//     }
// }

// /// Make a GhtForest trait with a force method that does the forcing+merging logic
// /// This trait will be recursive on the variadic of `Ght`s.
// pub trait GhtForest {
//     type Schema; // each element is a `GeneralizedHashTrieNode<Schema = Self::Schema>`

//     // ...
//     fn force_and_merge(self, key: !) -> !;
// }
