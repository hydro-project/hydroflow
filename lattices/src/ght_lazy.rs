use std::hash::Hash;
use std::marker::PhantomData;

use sealed::sealed;
use variadics::{
    var_args, var_expr, var_type, PartialEqVariadic, Split, SplitBySuffix, Variadic, VariadicExt,
};

use crate::ght::{GeneralizedHashTrieNode, GhtGet, GhtInner, GhtKey, GhtLeaf, GhtTakeLeaf};

// Remaining Questions
// 1. Should the first element in the forest be a single GhtLeaf?
//    Doesn't seem to help with columnstore case (its just the range [1..]) but it's that way in the paper.
// 2. How does "get" work?
//    - get on a leaf forces. That should work ok.
//    - get on an internal node "just get"s .. what's the analogy in the forest if get on an inner fails?
//      Could there be "previously forced siblings"? Do we need get tombstones/sibling pointers?
//      See Figure 11 in the paper: x0->b0 would be in the sibling. What do I do for get(0)?
// 3. Similarly, how does "iter" work? Where are "all the keys"?
// I think maybe we need to understand the context in which get and iter are called and figure out
// forest-appropriate outer algorithms

#[sealed]
/// COLT from Wang/Willsey/Suciu
pub trait ColumnLazyTrieNode: GeneralizedHashTrieNode {
    /// into_iter for leaf elements, needed by force below
    fn into_iter(self) -> Option<impl Iterator<Item = Self::Schema>>;

    /// pull all the data out of this trie node but retain the reference
    fn drain(&mut self) -> Option<impl Iterator<Item = Self::Schema>>;

    /// result of `force`ing a node
    type Force: GeneralizedHashTrieNode; // + GhtHasChildren;
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
    GhtLeaf<Schema, Rest>: GeneralizedHashTrieNode<Schema = Schema>,
    GhtInner<Head, GhtLeaf<Schema, Rest>>: GeneralizedHashTrieNode<Schema = Schema>,
{
    fn into_iter(self) -> Option<impl Iterator<Item = Self::Schema>> {
        Some(self.elements.into_iter())
    }

    fn drain(&mut self) -> Option<impl Iterator<Item = Self::Schema>> {
        Some(self.elements.drain())
    }
    // Node::Schema: SplitBySuffix<var_type!(Head, ...Node::SuffixSchema)>
    type Force = GhtInner<Head, GhtLeaf<Schema, Rest>>;
    fn force(mut self) -> Option<Self::Force> {
        let mut retval = Self::Force::default();
        self.forced = true;
        for row in self.into_iter().unwrap() {
            // let var_expr!(h, ...r) = row;
            // retval.insert((h, r));
            retval.insert(row);
        }
        Some(retval)
    }

    fn force_drain(&mut self) -> Option<GhtInner<Head, GhtLeaf<Schema, Rest>>> {
        let mut retval = Self::Force::default();
        self.forced = true;
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
    ($a:ty, $( $b:ty ),* => ()) => {
        var_type!(GhtType!($a, $( $b ),* => ()))
    };
    ($a:ty => $c:ty, $( $d:ty ),* ) => {
        (GhtType!($a => $c, $( $d ),*), GhtForestType!($a, $c => $( $d ),*))
    };
    ($a:ty, $( $b:ty ),* => $c:ty) => {
        (GhtType!($a, $( $b ),* => $c), GhtForestType!($a, $( $b ),*, $c => ()))
    };

    ($a:ty, $( $b:ty ),* => $c:ty, $( $d:ty ),* ) => {
        (GhtType!($a, $( $b ),* => $c, $( $d ),*), GhtForestType!($a, $( $b ),* , $c => $( $d ),*))
    };

    ($a:ty, $( $b:ty ),* ) => {
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

    // /// given a prefix search_key, return a list of node references,
    // /// one per trie.
    // fn get(&self, search_key: SearchKey) -> VariadicExt;
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
                    forced: false,
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

    // fn get<'a>(&self, search_key: SearchKey) -> VariadicExt {
    //     let var_expr!(first, ...rest) = self; //.forest;
    //     if first.height() < SearchKey::LEN {
    //         var_expr!((), <var_type!(TrieSecond, ...TrieRest) as GhtForest<SearchKey>>::get(rest, search_key))
    //     } else {

    //     }
    // }
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

/// Virtual COLT "node" with an API similar to GeneralizedHashTrieNode
/// Essentially a variadic of Option<&'_ Subtrie>
#[sealed]
pub trait ColtNode {
    // /// Schema variadic: the schema of the relation stored in this COLT.
    // /// This type is the same in all Tries and nodes of the COLT.
    type Schema: VariadicExt + Eq + Hash + Clone;
    /// SuffixSchema variadic: the suffix of the schema *from this node of the trie
    /// downward*. The first entry in this variadic is of type Head.
    /// This type is the same in all Tries of the COLT (but changes as we traverse downward)
    type SuffixSchema: VariadicExt + Eq + Hash + Clone;
    /// The type of the first column in the SuffixSchema
    /// This type is the same in all Tries of the COLT (but changes as we traverse downward)
    type Head: Eq + Hash;

    /// Type returned by [`Self::get`].
    type Get; //: ColtNode;

    /// On an Inner node, retrieves the value (child) associated with the given "head" key.
    /// returns an `Option` containing a reference to the value if found, or `None` if not found.
    /// On a Leaf node, returns None.
    fn get(&self, head: &GhtKey<Self::Head, Self::Schema>) -> Self::Get;
}

// // #[derive(RefCast)]
// // #[repr(transparent)]
// struct ColtNodeStruct<T>
// where
//     T: VariadicExt,
// {
//     forest: T,
// }

// Strategies:
// 1. GhtGet trait for GhtLeaf is different.
//        Problem: cannot call First::get and Rest::get with a single head.
//    In essence, we need First::Head = Rest::Head.
// 2. ColtNode returns different type for var_type!(GhtInner<Key, GhtLeaf<...>>) (drops First).
//        Problem: How?
// 3. Walk the tries in the forest sequentially (with a height parameter)
//        A bit less efficient/elegant
// 4. Somehow cast the prefix of Nones on each call to have Head = Rest::Head
//        Problem: How?

// #[sealed]
// impl<'a, First, Rest> ColtNode for var_type!(Option<&'a First>, ...Rest)
// where
//     First: GeneralizedHashTrieNode<
//             Head = Rest::Head,
//             SuffixSchema = Rest::SuffixSchema,
//             Schema = Rest::Schema,
//         > + GhtGet, // + GhtHasChildren,
//     Rest: ColtNode + VariadicExt,
//     Rest::SuffixSchema: VariadicExt + Split<var_type!(Rest::Head)>,
//     // First: GeneralizedHashTrieNode<Schema = Rest::Schema> + GhtGet, // + GhtHasChildren,
//     <First as GhtGet>::Get: 'a,
//     // var_type!(Option<&'a First::Get>, ...Rest::Get): ColtNode,
// {
//     type Schema = Rest::Schema;
//     type Head = Rest::Head;
//     type SuffixSchema = Rest::SuffixSchema;
//     // type Get = var_type!(Option<&'a First::Get>, ...Rest::Get);
//     type Get = var_type!(Option<&'a GhtLeaf<Self::Schema, Rest::SuffixSchema>>, ...Rest::Get);

//     fn get(&self, head: &GhtKey<Self::Head, Self::Schema>) -> Self::Get {
//         let (first, rest) = self;
//         if let Some(first) = *first {
//             if first.height() <= 1 {
//                 // We want the SuffixSchema to be set to the SuffixSchema of the first non-empty leaf
//                 // so that the Head is set properly.
//                 var_expr!(None, ...Rest::get(rest, head))
//             } else if let GhtKey::Head(_first_head) = head {
//                 var_expr!(<First as GhtGet>::get(first, head), ...Rest::get(rest, head))
//             } else {
//                 // probably shouldn't end up down here
//                 panic!();
//                 var_expr!(None, ...Rest::get(rest, head))
//             }
//         } else {
//             var_expr!(None, ...Rest::get(rest, head))
//         }
//     }
// }
#[sealed]
impl<'a, Head, Rest> ColtNode for var_type!(Option<&'a GhtLeaf<Rest::Schema, Rest::SuffixSchema>>, ...Rest)
where
    Rest: ColtNode<Head = Head>,
    Head: Eq + Hash,
{
    type Schema = Rest::Schema;
    type Head = Rest::Head;
    type SuffixSchema = Rest::SuffixSchema;
    type Get = var_type!(Option<&'a GhtLeaf<Self::Schema, Rest::SuffixSchema>>, ...Rest::Get);

    fn get(&self, head: &GhtKey<Self::Head, Self::Schema>) -> Self::Get {
        let (_first, rest) = self;
        var_expr!(None, ...Rest::get(rest, head))
    }
}

#[sealed]
impl<'a, Head, Head2, Rest, Node> ColtNode for var_type!(Option<&'a GhtInner<Head, GhtInner<Head2, Node>>>, ...Rest)
where
    Rest: ColtNode<Head = Head>,
    Head: Eq + Hash + Clone,
    Head2: Eq + Hash + Clone,
    Node: GeneralizedHashTrieNode,
    GhtInner<Head, GhtInner<Head2, Node>>: GeneralizedHashTrieNode<
            Head = Rest::Head,
            SuffixSchema = Rest::SuffixSchema,
            Schema = Rest::Schema,
        > + GhtGet,
    GhtInner<Head2, Node>: GeneralizedHashTrieNode<Schema = Rest::Schema> + GhtGet,
{
    type Schema = Rest::Schema;
    type Head = Rest::Head;
    type SuffixSchema = Rest::SuffixSchema;
    type Get =
        var_type!(Option<&'a <GhtInner<Head, GhtInner<Head2, Node>> as GhtGet>::Get>, ...Rest::Get);

    fn get(&self, head: &GhtKey<Self::Head, Self::Schema>) -> Self::Get {
        let (_first, rest) = self;
        if let Some(first) = self.0 {
            var_expr!(first.get(head), ...Rest::get(rest, head))
        } else {
            var_expr!(None, ...Rest::get(rest, head))
        }
    }
}
#[sealed]
impl<'a, Head, Rest, Schema, ValType> ColtNode for var_type!(Option<&'a GhtInner<Head, GhtLeaf<Schema, ValType>>>, ...Rest)
where
    Rest: ColtNode<Head = Head>,
    Head: Eq + Hash + Clone,
    // GhtInner<Head, GhtInner<Head2, Node>>: GeneralizedHashTrieNode<
    //         Head = Rest::Head,
    //         SuffixSchema = Rest::SuffixSchema,
    //         Schema = Rest::Schema,
    //     > + GhtGet,
    // GhtInner<Head2, Node>: GeneralizedHashTrieNode<Schema = Rest::Schema> + GhtGet,
    Schema: Eq + Hash + Clone + PartialEqVariadic,
    ValType: Eq + Hash + Clone + PartialEqVariadic,
    GhtLeaf<Schema, ValType>: GeneralizedHashTrieNode,
    Schema: 'static + Eq + VariadicExt + Hash + Clone + SplitBySuffix<ValType> + PartialEqVariadic,
    <Schema as SplitBySuffix<ValType>>::Prefix: Eq + Hash + Clone,
{
    type Schema = Rest::Schema;
    type Head = Rest::Head;
    type SuffixSchema = Rest::SuffixSchema;
    type Get = Rest::Get; // Option<&'a <GhtLeaf<Schema, ValType> as GhtGet>::Get>,

    fn get(&self, head: &GhtKey<Self::Head, Self::Schema>) -> Self::Get {
        let (_first, rest) = self;
        Rest::get(rest, head)
        // let (_first, rest) = self;
        // if let Some(first) = self.0 {
        //     var_expr!(first.get(head), ...Rest::get(rest, head))
        // } else {
        //     var_expr!(None, ...Rest::get(rest, head))
        // }
    }
}

#[sealed]
impl<'a, Head, Node> ColtNode for var_type!(Option<&'a GhtInner<Head, Node>>)
where
    GhtInner<Head, Node>: GeneralizedHashTrieNode + GhtGet,
    Head: Clone + Eq + Hash,
    Node: GeneralizedHashTrieNode,
{
    type Schema = <GhtInner<Head, Node> as GeneralizedHashTrieNode>::Schema;
    type SuffixSchema = <GhtInner<Head, Node> as GeneralizedHashTrieNode>::SuffixSchema;
    type Head = <GhtInner<Head, Node> as GeneralizedHashTrieNode>::Head;
    type Get = var_type!(Option<&'a <GhtInner<Head, Node> as GhtGet>::Get>);

    fn get(&self, head: &GhtKey<Self::Head, Self::Schema>) -> Self::Get {
        var_expr!(self.0.unwrap().get(head))
    }
}

// #[sealed]
// impl ColtNode for var_type!() {
//     type Head = ();
//     type Get = var_type!();
//     type Schema = ();
//     type SuffixSchema = ();

//     fn get(&self, _head: &Self::Head) -> Self::Get {
//         var_expr!()
//     }
// }
