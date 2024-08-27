use std::hash::Hash;
use std::marker::PhantomData;

use sealed::sealed;
use variadics::{var_expr, var_type, PartialEqVariadic, Split, SplitBySuffix, VariadicExt};

use crate::ght::{GeneralizedHashTrieNode, GhtHasChildren, GhtInner, GhtLeaf, GhtTakeLeaf};

#[sealed]
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

#[sealed]
impl<Head, Node> ColumnLazyTrieNode for GhtInner<Head, Node>
where
    Head: 'static + Hash + Eq + Clone,
    Node: 'static + Clone + ColumnLazyTrieNode,
    Node::Schema: SplitBySuffix<var_type!(Head, ...Node::SuffixSchema)>,
{
    fn into_iter(self) -> Option<impl Iterator<Item = Self::Schema>> {
        None::<Box<dyn Iterator<Item = Self::Schema>>>
    }

    fn drain(&mut self) -> Option<impl Iterator<Item = Self::Schema>> {
        None::<Box<dyn Iterator<Item = Self::Schema>>>
    }

    type Force = GhtInner<Head, Node> where Node:GeneralizedHashTrieNode;
    fn force(self) -> Option<Self::Force> {
        None
    }

    fn force_drain(&mut self) -> Option<GhtInner<Head, Node>> {
        None
    }
}

#[sealed]
impl<Schema, Head, Rest> ColumnLazyTrieNode for GhtLeaf<Schema, var_type!(Head, ...Rest)>
where
    Head: 'static + Clone + Hash + Eq,
    Rest: 'static + Clone + Hash + Eq + VariadicExt,
    Schema: 'static + Hash + Eq + Clone + VariadicExt + PartialEqVariadic,
    // for<'r, 's> <var_type!(Head, ...Rest) as VariadicExt>::AsRefVar<'r>:
    //     PartialEq<<var_type!(Head, ...Rest) as VariadicExt>::AsRefVar<'s>>,
    Rest: PartialEqVariadic,
    // for<'r> Rest::AsRefVar<'r>: PartialEq<Rest::AsRefVar<'r>>,
    // for<'r> Schema::AsRefVar<'r>: PartialEq<Schema::AsRefVar<'r>>,
    Schema: SplitBySuffix<var_type!(Head, ...Rest)>,
    Schema: SplitBySuffix<Rest>,
    <Schema as SplitBySuffix<(Head, Rest)>>::Prefix: Eq + Hash + Clone,
    <Schema as SplitBySuffix<Rest>>::Prefix: Eq + Hash + Clone,
{
    fn into_iter(self) -> Option<impl Iterator<Item = Self::Schema>> {
        Some(self.elements.into_iter())
    }

    fn drain(&mut self) -> Option<impl Iterator<Item = Self::Schema>> {
        Some(self.elements.drain())
    }
    // Node::Schema: SplitBySuffix<var_type!(Head, ...Node::SuffixSchema)>
    type Force = GhtInner<Head, GhtLeaf<Schema, Rest>>;
    fn force(self) -> Option<Self::Force> {
        let mut retval = Self::Force::default();
        for row in self.into_iter().unwrap() {
            // let var_expr!(h, ...r) = row;
            // retval.insert((h, r));
            retval.insert(row);
        }
        Some(retval)
    }

    fn force_drain(&mut self) -> Option<GhtInner<Head, GhtLeaf<Schema, Rest>>> {
        let mut retval = Self::Force::default();
        for row in self.drain().unwrap() {
            // let var_expr!(h, ...r) = row;
            // retval.insert((h, r));
            retval.insert(row);
        }
        Some(retval)
    }
}

#[macro_export]
/// Constructs a forest (variadic list) of Ght structs,
/// one for each height from 1 to length of the schema
macro_rules! GhtForestType {
    ($a:tt, $( $b:tt ),* => ()) => {
        var_type!(GhtType!($a, $( $b ),* => ()))
    };
    ($a:tt => $c:tt, $( $d:tt ),* ) => {
        (GhtType!($a => $c, $( $d ),*), GhtForestType!($a, $c => $( $d ),*))
    };
    ($a:tt, $( $b:tt ),* => $c:tt) => {
        (GhtType!($a, $( $b ),* => $c), GhtForestType!($a, $( $b ),*, $c => ()))
    };

    ($a:tt, $( $b:tt ),* => $c:tt, $( $d:tt ),* ) => {
        (GhtType!($a, $( $b ),* => $c, $( $d ),*), GhtForestType!($a, $( $b ),* , $c => $( $d ),*))
    };

    ($a:tt, $( $b:tt ),* ) => {
        GhtForestType!($a => $( $b ),*)
    };
}

/// Make a GhtForest trait with a force method that does the forcing+merging logic
/// This trait will be recursive on the variadic of `Ght`s.
#[sealed]
pub trait GhtForest<SearchKey>
where
    SearchKey: VariadicExt,
{
    /// tries to find a trie with a matching leaf.
    /// if it finds such a trie, and the search_key is longer than the height,
    /// it will force the leaf into the next trie over and recurse.
    /// returns true if it forces (1 or more times), and false otherwise.
    fn force(&mut self, search_key: SearchKey) -> bool;
}

// /// GhtForestStruct is a metadata node pointing to a variadic list of GHTs
// #[derive(RefCast)]
// #[repr(transparent)]
// pub struct GhtForestStruct<T>
// where
//     T: VariadicExt,
// {
//     pub(crate) forest: T,
// }

#[sealed]
impl<TrieFirst, TrieSecond, TrieRest, SearchKey /* , Head, Rest */> GhtForest<SearchKey> for var_type!(TrieFirst, TrieSecond, ...TrieRest)
where
    TrieFirst: GeneralizedHashTrieNode + GhtTakeLeaf,
    TrieSecond: GeneralizedHashTrieNode<Schema = TrieFirst::Schema> + GhtTakeLeaf,
    SearchKey: VariadicExt + Split<TrieFirst::Schema> + Clone,
    var_type!(TrieSecond, ...TrieRest): GhtForest<SearchKey>,
    // GhtForestStruct<var_type!(TrieSecond, ...TrieRest)>: GhtForest<SearchKey>,
    TrieFirst::Schema: PartialEqVariadic + SplitBySuffix<TrieFirst::ValType> + Eq + Hash + Clone,
    TrieSecond::Schema: PartialEqVariadic + SplitBySuffix<TrieSecond::ValType> + Eq + Hash + Clone,
    Self: ForestFindLeaf<TrieFirst::Schema>,
    <<TrieFirst::Schema as VariadicExt>::Reverse as VariadicExt>::Reverse: Eq + Hash + Clone,
    GhtLeaf<
        <TrieFirst as GeneralizedHashTrieNode>::Schema,
        <TrieFirst as GeneralizedHashTrieNode>::ValType,
    >: ColumnLazyTrieNode,
{
    fn force<'a>(&mut self, search_key: SearchKey) -> bool {
        let var_expr!(first, ...rest) = self; //.forest;
        if first.height() < SearchKey::LEN {
            let (row, _): (
                TrieFirst::Schema,
                <SearchKey as Split<<TrieFirst as GeneralizedHashTrieNode>::Schema>>::Suffix,
            ) = search_key.clone().split();
            // try to force first
            if let Some(leaf) = first.take_containing_leaf(row.as_ref_var()) {
                let var_expr!(rest_first, ..._rr) = rest;
                // TrieFirst::ValType IS NOT the same as TrieSecond::ValType,
                // but the elements in the leaf are the same.
                // So we just need a new GhtLeaf with the right ValType.
                let leaf = GhtLeaf::<TrieSecond::Schema, TrieSecond::ValType> {
                    elements: leaf.elements,
                    _suffix_schema: PhantomData,
                };
                rest_first.merge_leaf(row.as_ref_var(), leaf);
                // drop through and recurse: we may have to force again in the neighbor
            }
            // recurse
            <var_type!(TrieSecond, ...TrieRest) as GhtForest<SearchKey>>::force(rest, search_key)
        } else {
            false
        }
    }
}

/// If we're on the last trie in the forest, there's nowhere to force right to
#[sealed]
impl<SearchKey, TrieFirst> GhtForest<SearchKey> for var_type!(TrieFirst)
where
    SearchKey: VariadicExt,
    TrieFirst: GeneralizedHashTrieNode,
{
    fn force<'a>(&mut self, _search_key: SearchKey) -> bool {
        false
    }
}

#[sealed]
impl<SearchKey> GhtForest<SearchKey> for var_type!()
where
    SearchKey: VariadicExt,
{
    fn force<'a>(&mut self, _search_key: SearchKey) -> bool {
        false
    }
}

#[sealed]
/// a trait for finding a matching leaf in the forest
pub trait ForestFindLeaf<Schema>
where
    Schema: Eq + Hash + VariadicExt + PartialEqVariadic,
{
    /// find a matching leaf in the forest
    fn find_containing_leaf(&self, row: Schema::AsRefVar<'_>) -> Option<&'_ GhtLeaf<Schema, ()>>;
}

#[sealed]
impl<TrieFirst, TrieRest> ForestFindLeaf<<TrieFirst as GeneralizedHashTrieNode>::Schema> for var_type!(TrieFirst, ...TrieRest)
where
    <TrieFirst as GeneralizedHashTrieNode>::Schema: PartialEqVariadic,
    TrieFirst: GeneralizedHashTrieNode,
    TrieRest: ForestFindLeaf<<TrieFirst as GeneralizedHashTrieNode>::Schema>,
{
    fn find_containing_leaf(
        &self,
        row: <<TrieFirst as GeneralizedHashTrieNode>::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<<TrieFirst as GeneralizedHashTrieNode>::Schema, ()>> {
        let var_expr!(first, ...rest) = &self;
        if let Some(leaf) = first.find_containing_leaf(row) {
            // TODO!!!!
            unsafe {
                std::mem::transmute::<
                    &GhtLeaf<
                        <TrieFirst as GeneralizedHashTrieNode>::Schema,
                        <TrieFirst as GeneralizedHashTrieNode>::ValType,
                    >,
                    Option<&GhtLeaf<<TrieFirst as GeneralizedHashTrieNode>::Schema, ()>>,
                >(leaf)
            }
        } else {
            rest.find_containing_leaf(row)
            // let remainder = GhtForestStruct::<TrieRest>::ref_cast(rest);
            // <GhtForestStruct<TrieRest> as ForestFindLeaf<Schema, ValType>>::find_containing_leaf(
            //     remainder, row,
            // )
        }
    }
}

#[sealed]
impl<Schema> ForestFindLeaf<Schema> for var_type!()
where
    Schema: Eq + Hash + VariadicExt + PartialEqVariadic,
{
    fn find_containing_leaf(
        &self,
        _row: <Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<Schema, ()>> {
        None
    }
}

// impl<TrieFirst, TrieRest> Default for GhtForestStruct<var_type!(TrieFirst, ...TrieRest)>
// where
//     // T: VariadicExt,
//     TrieFirst: Default + GeneralizedHashTrieNode,
//     TrieRest: VariadicExt,
//     GhtForestStruct<TrieRest>: Default,
//     // for<'a> <TrieRest as VariadicExt>::AsRefVar<'a>: PartialEq,
//     // <TrieFirst as GhtHasChildren>::Node: Eq + Hash,
//     // GhtLeaf<TrieFirst::Node>: GeneralizedHashTrieNode,
//     // need something like TrieFirst::Schema = TrieRest.0::Schema?
// {
//     fn default() -> Self {
//         let first_trie = TrieFirst::default();
//         let rest = GhtForestStruct::<TrieRest>::default();
//         let rest_forest: TrieRest = rest.forest;

//         Self {
//             forest: var_expr!(first_trie, ...rest_forest),
//         }
//     }
// }

// impl Default for GhtForestStruct<var_type!()> {
//     fn default() -> Self {
//         Self {
//             forest: var_expr!(),
//         }
//     }
// }
