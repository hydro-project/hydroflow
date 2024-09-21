use std::hash::Hash;
use std::marker::PhantomData;

use sealed::sealed;
use variadics::{var_expr, var_type, PartialEqVariadic, Split, SplitBySuffix, VariadicExt};

use crate::ght::{GeneralizedHashTrieNode, GhtGet, GhtInner, GhtLeaf, GhtTakeLeaf, TupleSet};

// Remaining Questions
// 1. Should the first element in the forest be a single GhtLeaf?
//    Doesn't seem to help with columnstore case (its just the range [1..]) but it's that way in the paper.
// 2. How does "get" work?
//    - get on a leaf forces. That should work ok.
//    - get on an internal node "just get"s .. what's the analogy in the forest if get on an inner fails?
//      Could there be "previously forced siblings"? Do we need get tombstones/sibling pointers?
//      See Figure 11 in the paper: x0->b0 would be in the sibling. What do I do for get(0)?
// 3. Similarly, how does "iter" work? Where are "all the keys"?

// Strategies for Get on a ColtNode:
// 1. ColtNode returns different type for var_type!(GhtInner<Key, GhtLeaf<...>>) (drops First).
//        THIS IS WHAT WE IMPLEMENTED
// 2. Somehow cast the prefix of Nones on each call to have Head = ColtNode::Reverse::Car::Head
//        Problem: messy
// 3. Walk the tries in the forest sequentially (with a height parameter)
//        Seems less efficient/elegant

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
impl<Schema, Head, Rest, Storage> ColumnLazyTrieNode
    for GhtLeaf<Schema, var_type!(Head, ...Rest), Storage>
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
    Storage: TupleSet<Schema = Schema>
        + Default // + Iterator<Item = Schema>
        + IntoIterator<Item = Schema>,
    GhtLeaf<Schema, Rest, Storage>: GeneralizedHashTrieNode<Schema = Schema>,
    GhtInner<Head, GhtLeaf<Schema, Rest, Storage>>: GeneralizedHashTrieNode<Schema = Schema>,
{
    fn into_iter(self) -> Option<impl Iterator<Item = Self::Schema>> {
        Some(self.elements.into_iter())
    }

    fn drain(&mut self) -> Option<impl Iterator<Item = Self::Schema>> {
        Some(self.elements.drain())
    }
    // Node::Schema: SplitBySuffix<var_type!(Head, ...Node::SuffixSchema)>
    type Force = GhtInner<Head, GhtLeaf<Schema, Rest, Storage>>;
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

    fn force_drain(&mut self) -> Option<GhtInner<Head, GhtLeaf<Schema, Rest, Storage>>> {
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
        (GhtType!(() => $a, $( $b ),*), GhtForestType!($a => $( $b ),*))
    };
}

/// Virtual COLT "node" with an API similar to GeneralizedHashTrieNode.
/// This is basically a GhtForest of subtries, with each subtrie wrapped
/// in Option<&'_ Subtrie>. The Option is there because the `get`s from the root(s)
/// that led us to these subtries may have encountered no match in some of the tries
/// in the forest; those tries are None now.
#[sealed]
pub trait ColtNode {
    /// Schema variadic: the schema of the relation stored in this COLT.
    /// This type is the same in all Tries and nodes of the COLT.
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
    fn get(self, head: &Self::Head) -> Self::Get;
}

/// `ColtNode` without the first (head) trie.
#[sealed]
pub trait ColtNodeTail<InnerToMerge>: ColtNode {
    /// merge an inner node into the head of this tail of the forest
    fn merge(&mut self, inner_to_merge: InnerToMerge);
}

#[sealed]
impl<'a, Rest, Schema, SuffixSchema, Storage> ColtNode for var_type!(&'a mut GhtLeaf<Schema, SuffixSchema, Storage>, ...Rest)
where
    Rest: ColtNodeTail<
        <GhtLeaf<Schema, SuffixSchema, Storage> as ColumnLazyTrieNode>::Force,
        Schema = Schema,
        // SuffixSchema = SuffixSchema,
    >,
    <Rest as ColtNode>::SuffixSchema: 'a,
    GhtLeaf<Schema, SuffixSchema, Storage>: ColumnLazyTrieNode,
    Schema: Clone + Hash + Eq + VariadicExt,
    SuffixSchema: Clone + Hash + Eq + VariadicExt,
    Storage: TupleSet<Schema = Schema>,
{
    type Schema = Schema;
    type Head = Rest::Head;
    type SuffixSchema = SuffixSchema;
    type Get = Rest::Get;

    fn get(self, head: &Self::Head) -> Self::Get {
        let (first, mut rest) = self;
        let forced = first.force_drain().unwrap();
        ColtNodeTail::merge(&mut rest, forced);
        Rest::get(rest, head)
    }
}
#[sealed]
impl<'a, Rest, Schema, SuffixSchema, T, Storage> ColtNodeTail<T> for var_type!(&'a mut GhtLeaf<Schema, SuffixSchema, Storage>, ...Rest)
where
    Rest: ColtNodeTail<
        <GhtLeaf<Schema, SuffixSchema, Storage> as ColumnLazyTrieNode>::Force,
        Schema = Schema,
        // SuffixSchema = SuffixSchema,
    >,
    <Rest as ColtNode>::SuffixSchema: 'a,
    GhtLeaf<Schema, SuffixSchema, Storage>: ColumnLazyTrieNode,
    Schema: Clone + Hash + Eq + VariadicExt,
    SuffixSchema: Clone + Hash + Eq + VariadicExt,
    Storage: TupleSet<Schema = Schema>,
{
    fn merge(&mut self, _inner_to_merge: T) {
        panic!();
    }
}

#[sealed]
impl<'a, Head, Head2, Rest, Node> ColtNode for var_type!(&'a mut GhtInner<Head, GhtInner<Head2, Node>>, ...Rest)
where
    Rest: ColtNode<Head = Head>,
    Head: Eq + Hash + Clone,
    Head2: Eq + Hash + Clone,
    Node: GeneralizedHashTrieNode,
    GhtInner<Head, GhtInner<Head2, Node>>: GeneralizedHashTrieNode<
        Head = Rest::Head,
        SuffixSchema = Rest::SuffixSchema,
        Schema = Rest::Schema,
    >,
    GhtInner<Head2, Node>: GeneralizedHashTrieNode<Schema = Rest::Schema>,
{
    type Schema = Rest::Schema;
    type Head = Rest::Head;
    type SuffixSchema = Rest::SuffixSchema;
    type Get = var_type!(&'a mut GhtInner<Head2, Node>, ...Rest::Get);

    fn get(self, head: &Self::Head) -> Self::Get {
        let (first, rest) = self;
        // create a child entry here for this get, to absorb future forces
        // TODO(mingwei): extra clone here if entry already exists.
        let child = first.children.entry(head.clone()).or_default();
        var_expr!(child, ...Rest::get(rest, head))
    }
}
#[sealed]
impl<'a, Head, Head2, Rest, Node, T> ColtNodeTail<T> for var_type!(&'a mut GhtInner<Head, GhtInner<Head2, Node>>, ...Rest)
where
    Rest: ColtNode<Head = Head>,
    Head: Eq + Hash + Clone,
    Head2: Eq + Hash + Clone,
    Node: GeneralizedHashTrieNode,
    GhtInner<Head, GhtInner<Head2, Node>>: GeneralizedHashTrieNode<
            Head = Rest::Head,
            SuffixSchema = Rest::SuffixSchema,
            Schema = Rest::Schema,
        > + GhtGet<Get = GhtInner<Head2, Node>>,
    GhtInner<Head2, Node>: GeneralizedHashTrieNode<Schema = Rest::Schema> + GhtGet,
{
    fn merge(&mut self, _inner_to_merge: T) {
        panic!();
    }
}

#[sealed]
impl<'a, Head, Rest, Schema, ValType, Storage> ColtNode for var_type!(&'a mut GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>, ...Rest)
where
    Rest: ColtNode<Head = Head>,
    Head: Eq + Hash + Clone,
    Schema: Eq + Hash + Clone + PartialEqVariadic,
    ValType: Eq + Hash + Clone + PartialEqVariadic,
    Storage: TupleSet<Schema = Schema>,
    GhtLeaf<Schema, ValType, Storage>: GeneralizedHashTrieNode,
    Schema: 'static + Eq + VariadicExt + Hash + Clone + SplitBySuffix<ValType> + PartialEqVariadic,
    <Schema as SplitBySuffix<ValType>>::Prefix: Eq + Hash + Clone,
    GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>:
        GeneralizedHashTrieNode<Head = Head> + GhtGet,
    // Rest: ColtNode<Head = Head>,
    // Head: Eq + Hash + Clone,
    // Head2: Eq + Hash + Clone,
    // Node: GeneralizedHashTrieNode,
    GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>: GeneralizedHashTrieNode<
        Head = Rest::Head,
        // SuffixSchema = Rest::SuffixSchema,
        Schema = Rest::Schema,
    >,
    GhtLeaf<Schema, ValType, Storage>: GeneralizedHashTrieNode<Schema = Rest::Schema> + GhtGet,
{
    type Schema = Rest::Schema;
    type Head = Rest::Head;
    type SuffixSchema = Rest::SuffixSchema;
    // type Get = Rest::Get; // Option<&'a <GhtLeaf<Schema, ValType> as GhtGet>::Get>,
    type Get = var_type!(&'a mut GhtLeaf<Schema, ValType, Storage>, ...Rest::Get);

    fn get(self, head: &Self::Head) -> Self::Get {
        let (first, rest) = self;
        let child = first.children.entry(head.clone()).or_default();
        var_expr!(child, ...Rest::get(rest, head))
    }
}
#[sealed]
impl<'a, Head, Rest, Schema, ValType, Storage>
    ColtNodeTail<GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>> for var_type!(&'a mut GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>, ...Rest)
where
    Rest: ColtNode<Head = Head, Schema = Schema>,
    Head: Eq + Hash + Clone,
    Schema: Eq + Hash + Clone + PartialEqVariadic,
    ValType: Eq + Hash + Clone + PartialEqVariadic,
    Storage: TupleSet<Schema = Schema>,
    GhtLeaf<Schema, ValType, Storage>: GeneralizedHashTrieNode<Schema = Schema>,
    Schema: 'static + Eq + VariadicExt + Hash + Clone + SplitBySuffix<ValType> + PartialEqVariadic,
    <Schema as SplitBySuffix<ValType>>::Prefix: Eq + Hash + Clone,
    GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>: GeneralizedHashTrieNode<Head = Head, Schema = Schema>
        + crate::Merge<GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>>
        + GhtGet,
{
    fn merge(&mut self, inner_to_merge: GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>) {
        // This shouldn't be none? IDK
        let (head, _rest) = self;
        crate::Merge::merge(*head, inner_to_merge);
    }
}

#[sealed]
impl<'a, Head, Node> ColtNode for var_type!(&'a mut GhtInner<Head, Node>)
where
    GhtInner<Head, Node>: GeneralizedHashTrieNode,
    Head: Clone + Eq + Hash,
    Node: GeneralizedHashTrieNode,
{
    type Schema = <GhtInner<Head, Node> as GeneralizedHashTrieNode>::Schema;
    type SuffixSchema = <GhtInner<Head, Node> as GeneralizedHashTrieNode>::SuffixSchema;
    type Head = Head;
    type Get = var_type!(&'a mut Node);

    fn get(self, head: &Self::Head) -> Self::Get {
        let child = self.0.children.entry(head.clone()).or_default();
        var_expr!(child)
    }
}
#[sealed]
impl<'a, Head, Schema, ValType, Storage>
    ColtNodeTail<GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>>
    for var_type!(&'a mut GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>)
where
    GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>: GeneralizedHashTrieNode<Head = Head>
        + GhtGet
        + crate::Merge<GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>>
        + GhtGet,
    GhtLeaf<Schema, ValType, Storage>: GeneralizedHashTrieNode<Schema = Schema>,
    Head: Clone + Eq + Hash,
    Schema: Clone + Eq + Hash + VariadicExt,
    Storage: TupleSet<Schema = Schema>,
{
    fn merge(&mut self, inner_to_merge: GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>) {
        crate::Merge::merge(self.0, inner_to_merge);
    }
}

// #[sealed]
// impl<'a, Schema, ValType> ColtNode for var_type!(Option<&'a mut GhtLeaf<Schema, ValType>>)
// where
//     GhtLeaf<Schema, ValType>: GeneralizedHashTrieNode + GhtGet,
//     ValType: Clone + Eq + Hash,
// {
//     type Schema = Schema;
//     type SuffixSchema = ValType;
//     type Head = <GhtInner<Head, Node> as GeneralizedHashTrieNode>::Head;
//     type Get = var_type!(Option<&'a mut <GhtInner<Head, Node> as GhtGet>::Get>);

//     fn get_broken(self, head: &GhtKey<Self::Head, Self::Schema>) -> Self::Get {
//         var_expr!(self.0.unwrap().get_mut(head))
//     }

//     fn get(self, head: &GhtKey<Self::Head, Self::Schema>) -> Self::Get {
//         // var_expr!(self.0.unwrap().get_mut(head))
//         var_expr!(self.0.and_then(|x| x.get_mut(head)))
//     }
// }

// THE CODE BELOW MAY NOT BE USEFUL
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

#[sealed]
impl<TrieFirst, TrieSecond, TrieRest, SearchKey /* , Head, Rest */> GhtForest<SearchKey> for var_type!(TrieFirst, TrieSecond, ...TrieRest)
where
    TrieFirst: GeneralizedHashTrieNode + GhtTakeLeaf,
    TrieSecond: GeneralizedHashTrieNode<Schema = TrieFirst::Schema, Storage = TrieFirst::Storage>
        + GhtTakeLeaf,
    SearchKey: VariadicExt + Split<TrieFirst::Schema> + Clone,
    var_type!(TrieSecond, ...TrieRest): GhtForest<SearchKey>,
    // GhtForestStruct<var_type!(TrieSecond, ...TrieRest)>: GhtForest<SearchKey>,
    TrieFirst::Schema: PartialEqVariadic + SplitBySuffix<TrieFirst::ValType> + Eq + Hash + Clone,
    TrieSecond::Schema: PartialEqVariadic + SplitBySuffix<TrieSecond::ValType> + Eq + Hash + Clone,
    Self: ForestFindLeaf<TrieFirst::Schema, TrieFirst::Storage>,
    <<TrieFirst::Schema as VariadicExt>::Reverse as VariadicExt>::Reverse: Eq + Hash + Clone,
    GhtLeaf<
        <TrieFirst as GeneralizedHashTrieNode>::Schema,
        <TrieFirst as GeneralizedHashTrieNode>::ValType,
        TrieFirst::Storage,
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
                let leaf = GhtLeaf::<TrieSecond::Schema, TrieSecond::ValType, TrieSecond::Storage> {
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
pub trait ForestFindLeaf<Schema, Storage>
where
    Schema: Eq + Hash + VariadicExt + PartialEqVariadic,
    Storage: TupleSet<Schema = Schema>,
{
    /// find a matching leaf in the forest
    fn find_containing_leaf(
        &self,
        row: Schema::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<Schema, (), Storage>>;
}

#[sealed]
impl<TrieFirst, TrieRest> ForestFindLeaf<TrieFirst::Schema, TrieFirst::Storage> for var_type!(TrieFirst, ...TrieRest)
where
    <TrieFirst as GeneralizedHashTrieNode>::Schema: PartialEqVariadic,
    TrieFirst: GeneralizedHashTrieNode,
    TrieRest: ForestFindLeaf<<TrieFirst as GeneralizedHashTrieNode>::Schema, TrieFirst::Storage>,
{
    fn find_containing_leaf(
        &self,
        row: <<TrieFirst as GeneralizedHashTrieNode>::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<TrieFirst::Schema, (), TrieFirst::Storage>> {
        let var_expr!(first, ...rest) = &self;
        if let Some(leaf) = first.find_containing_leaf(row) {
            // TODO!!!!
            unsafe {
                std::mem::transmute::<
                    &GhtLeaf<TrieFirst::Schema, TrieFirst::ValType, TrieFirst::Storage>,
                    Option<&GhtLeaf<TrieFirst::Schema, (), TrieFirst::Storage>>,
                >(leaf)
            }
        } else {
            rest.find_containing_leaf(row)
        }
    }
}

#[sealed]
impl<Schema, Storage> ForestFindLeaf<Schema, Storage> for var_type!()
where
    Schema: Eq + Hash + VariadicExt + PartialEqVariadic,
    Storage: TupleSet<Schema = Schema>,
{
    fn find_containing_leaf(
        &self,
        _row: <Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<Schema, (), Storage>> {
        None
    }
}
