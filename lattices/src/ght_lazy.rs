use std::hash::Hash;
use std::marker::PhantomData;

use sealed::sealed;
use variadics::variadic_collections::VariadicCollection;
use variadics::variadic_collections::VariadicCollection;
use variadics::{
    var_args, var_expr, var_type, PartialEqVariadic, Split, SplitBySuffix, VariadicExt,
};

use crate::ght::{GeneralizedHashTrieNode, GhtGet, GhtInner, GhtLeaf};

#[sealed]
/// COLT from Wang/Willsey/Suciu
///
/// In the paper, the COLT is an unbalanced trie that "grows upward" from leaves via the
/// `force` method. Unbalanced tries don't interact well with our Rust types, which want
/// a node's type to be defined via the type of its children, recursively --
/// that means all paths need to be the same length!
///
/// To work around this, our COLT is a forest of balanced GHTs of increasing height, from 0 to the number of columns
/// in the key - 1. Instead of the `force` method lengthening the path above a leaf by 1, it
/// instead `take`s the leaf from the current trie and merges it in to the trie to the right which
/// is 1 taller.

/// A trait for the special behavior we need from the GHTs in a COLT forest#[derive(Debug, Clone)]
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
    Storage: VariadicCollection<Schema = Schema>
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
        for row in self.elements.drain() {
            // let var_expr!(h, ...r) = row;
            // retval.insert((h, r));
            retval.insert(row);
        }
        Some(retval)
    }
}

#[macro_export]
/// Constructs a forest (variadic list) of Ght structs,
/// one for each height from 0 to length of the schema - 1
macro_rules! GhtForestType {
    // 1 => 0
    ($a:ty => ()) => {
        var_type!($crate::GhtType!($a => (): Column))
    };
    // 1 => 1
    ($a:ty => $c:ty ) => {
        ($crate::GhtType!($a => $c: Column), GhtForestType!($a, $c => ()))
    };
    // 1 => >1
    ($a:ty => $c:ty, $( $d:ty ),* ) => {
        ($crate::GhtType!($a => $c, $( $d ),*: Column), GhtForestType!($a, $c => $( $d ),*))
    };
    // >1 => 0
    ($a:ty, $( $b:ty ),* => ()) => {
        var_type!($crate::GhtType!($a, $( $b ),* => (): Column))
    };
    // >1 => 1
    ($a:ty, $( $b:ty ),* => $c:ty) => {
        ($crate::GhtType!($a, $( $b ),* => $c: Column), GhtForestType!($a, $( $b ),*, $c => ()))
    };
    // >1 => >1
    ($a:ty, $( $b:ty ),* => $c:ty, $( $d:ty ),* ) => {
        ($crate::GhtType!($a, $( $b ),* => $c, $( $d ),*: Column), GhtForestType!($a, $( $b ),* , $c => $( $d ),*))
    };
    // general 1
    ($a:ty) => {
        ($crate::GhtType!(() => $a: Column), GhtForestType!($a => ()))
    };
    // general >1
    ($a:ty, $( $b:ty ),* ) => {
        ($crate::GhtType!(() => $a, $( $b ),*: Column), GhtForestType!($a => $( $b ),*))
    };
}

/// Trait for taking a matching leaf out of a trie, as part of force method for COLTs
#[sealed]
pub trait GhtTakeLeaf: GeneralizedHashTrieNode {
    /// take a matching leaf out of a trie, as part of the force method for COLTs
    fn take_containing_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<GhtLeaf<Self::Schema, Self::ValType, Self::Storage>>;

    /// Merges a leaf into the hash trie
    fn merge_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
        leaf: GhtLeaf<Self::Schema, Self::ValType, Self::Storage>,
    ) -> bool;
}

#[sealed]
impl<Head, Head2, GrandNode> GhtTakeLeaf for GhtInner<Head, GhtInner<Head2, GrandNode>>
where
    Head: 'static + Hash + Eq + Clone,
    Head2: 'static + Hash + Eq + Clone,
    GrandNode: 'static + GeneralizedHashTrieNode,
    GhtInner<Head2, GrandNode>: GhtTakeLeaf,
    <GhtInner<Head2, GrandNode> as GeneralizedHashTrieNode>::Schema: SplitBySuffix<
        var_type!(Head,  ...<GhtInner<Head2, GrandNode> as GeneralizedHashTrieNode>::SuffixSchema),
    >,
{
    fn take_containing_leaf(
        &mut self,
        row: <<GhtInner<Head2, GrandNode> as GeneralizedHashTrieNode>::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<
        GhtLeaf<
            <GhtInner<Head2, GrandNode> as GeneralizedHashTrieNode>::Schema,
            <GhtInner<Head2, GrandNode> as GeneralizedHashTrieNode>::ValType,
            <GhtInner<Head2, GrandNode> as GeneralizedHashTrieNode>::Storage,
        >,
    > {
        let (_prefix, var_args!(head, ..._rest)) = Self::Schema::split_by_suffix_ref(row);
        self.children
            .get_mut(head)
            .and_then(|child| child.take_containing_leaf(row))
    }

    fn merge_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
        leaf: GhtLeaf<Self::Schema, Self::ValType, Self::Storage>,
    ) -> bool {
        // TODO(mingwei): clones head...
        let (_prefix, var_args!(head, ..._rest)) = Self::Schema::split_by_suffix_ref(row);
        let retval = self
            .children
            .entry(head.clone())
            .or_default()
            .merge_leaf(row, leaf);
        // self.height = self.get(head).and_then(|n| n.old_height).map(|h| h + 1);
        retval
    }
}

#[sealed]
impl<Head, Schema, ValType, Storage> GhtTakeLeaf
    for GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>
where
    Schema: 'static
        + Hash
        + Clone
        + Eq
        + PartialEqVariadic
        + SplitBySuffix<ValType>
        + SplitBySuffix<var_type!(Head, ...ValType)>,
    ValType: 'static + Hash + Clone + Eq + VariadicExt + PartialEqVariadic,
    Head: 'static + Hash + Eq + Clone,
    <Schema as SplitBySuffix<ValType>>::Prefix: Eq + Hash + Clone,
    // <Self as GeneralizedHashTrieNode>::Schema:
    //     SplitBySuffix<<Self as GeneralizedHashTrieNode>::SuffixSchema>,
    GhtLeaf<Schema, ValType, Storage>:
        GeneralizedHashTrieNode<Schema = Schema, ValType = ValType, Storage = Storage>,
    // Schema: SplitBySuffix<<Self as GeneralizedHashTrieNode>::SuffixSchema>,
    Schema: SplitBySuffix<(
        Head,
        <GhtLeaf<Schema, ValType, Storage> as GeneralizedHashTrieNode>::SuffixSchema,
    )>,
    Storage: 'static + VariadicCollection<Schema = Schema> + Default + Extend<Schema> + IntoIterator<Item = Schema>,
{
    fn take_containing_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<GhtLeaf<Self::Schema, Self::ValType, Self::Storage>> {
        let (_prefix, var_args!(head, ..._rest)) =
            <<Self as GeneralizedHashTrieNode>::Schema as SplitBySuffix<
                <Self as GeneralizedHashTrieNode>::SuffixSchema,
            >>::split_by_suffix_ref(row);
        let retval = self.children.remove(head);
        let new_child = GhtLeaf::<Schema, ValType, Storage> {
            elements: Default::default(),
            forced: true,
            _suffix_schema: Default::default(),
        };
        self.children.insert(head.clone(), new_child);
        retval
    }

    fn merge_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
        leaf: GhtLeaf<Self::Schema, Self::ValType, Self::Storage>,
    ) -> bool {
        let (_prefix, var_args!(head, ..._rest)) =
            <<Self as GeneralizedHashTrieNode>::Schema as SplitBySuffix<
                <Self as GeneralizedHashTrieNode>::SuffixSchema,
            >>::split_by_suffix_ref(row);
        if let Some(old_val) = self.children.get_mut(&head.clone()) {
            old_val.elements.extend(leaf.elements);
            true
        } else {
            self.children.insert(head.clone(), leaf);
            true
        }
    }
}

#[sealed]
impl<Schema, ValType, Storage> GhtTakeLeaf for GhtLeaf<Schema, ValType, Storage>
where
    Schema: 'static + Hash + Clone + Eq + PartialEqVariadic + SplitBySuffix<ValType>,
    ValType: 'static + Hash + Clone + Eq + VariadicExt + PartialEqVariadic,
    GhtLeaf<Schema, ValType, Storage>: GeneralizedHashTrieNode,
    Storage: 'static + VariadicCollection<Schema = Schema>,
{
    fn take_containing_leaf(
        &mut self,
        _row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<GhtLeaf<Self::Schema, Self::ValType, Self::Storage>> {
        None
    }

    fn merge_leaf(
        &mut self,
        _row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
        _leaf: GhtLeaf<Self::Schema, Self::ValType, Self::Storage>,
    ) -> bool {
        false
    }
}

/// Virtual COLT "node" with an API similar to GeneralizedHashTrieNode.
/// 
/// The "current node" at depth d from the root in the Wang/Willsey/Suciu
/// paper sits above an unbalanced trie. We emulate that here via a variadic
/// of nodes at depth d in all the tries in the forest (that are height d or taller.)
/// This is basically a GhtForest of subtries.
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
       // Schema = Schema,
        // SuffixSchema = SuffixSchema,
    >,
    <Rest as ColtNode>::SuffixSchema: 'a,
    GhtLeaf<Schema, SuffixSchema, Storage>: ColumnLazyTrieNode,
    Schema: Clone + Hash + Eq + VariadicExt,
    SuffixSchema: Clone + Hash + Eq + VariadicExt,
    Storage: VariadicCollection<Schema = Schema>,
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
    // Rest: ColtNodeTail<
    //     <GhtLeaf<Schema, SuffixSchema, Storage> as ColumnLazyTrieNode>::Force,
    //     // Schema = Schema,
    //     // SuffixSchema = SuffixSchema,
    // >,
    // <Rest as ColtNode>::SuffixSchema: 'a,
    // GhtLeaf<Schema, SuffixSchema, Storage>: ColumnLazyTrieNode,
    Schema: Clone + Hash + Eq + VariadicExt,
    // SuffixSchema: Clone + Hash + Eq + VariadicExt,
    Storage: VariadicCollection<Schema = Schema>,
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
    Storage: VariadicCollection<Schema = Schema>,
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
    Storage: VariadicCollection<Schema = Schema>,
    GhtLeaf<Schema, ValType, Storage>: GeneralizedHashTrieNode<Schema = Schema>,
    Schema: 'static + Eq + VariadicExt + Hash + Clone + SplitBySuffix<ValType> + PartialEqVariadic,
    <Schema as SplitBySuffix<ValType>>::Prefix: Eq + Hash + Clone,
    GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>: GeneralizedHashTrieNode<Head = Head, Schema = Schema>
        // can't use Merge with COLT bc columnstore is not a lattice!!
        // + crate::Merge<GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>>
        + GhtGet,
{
    fn merge(&mut self, inner_to_merge: GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>) {
        let (head, _rest) = self;
        // can't use Merge with COLT bc columnstore is not a lattice!!
        head.merge_node(inner_to_merge);
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
        // + crate::Merge<GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>>
        + GhtGet,
    GhtLeaf<Schema, ValType, Storage>: GeneralizedHashTrieNode<Schema = Schema>,
    Head: Clone + Eq + Hash,
    Schema: Clone + Eq + Hash + VariadicExt,
    Storage: VariadicCollection<Schema = Schema>,
{
    fn merge(&mut self, inner_to_merge: GhtInner<Head, GhtLeaf<Schema, ValType, Storage>>) {
        let (head, _rest) = self;
        // can't use Merge with COLT bc columnstore is not a lattice!!
        head.merge_node(inner_to_merge);
    }
}

// THE CODE BELOW MAY NOT BE USEFUL
/// A GhtForest trait with a force method that does the forcing+merging logic
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
    fn find_and_force(&mut self, search_key: SearchKey) -> bool;
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
    fn find_and_force<'a>(&mut self, search_key: SearchKey) -> bool {
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
                let new_leaf = GhtLeaf::<TrieSecond::Schema, TrieSecond::ValType, TrieSecond::Storage> {
                    elements: leaf.elements,
                    forced: false,
                    _suffix_schema: PhantomData,
                };
                rest_first.merge_leaf(row.as_ref_var(), new_leaf);
                // drop through and recurse: we may have to force again in the neighbor
            }
            // recurse
            <var_type!(TrieSecond, ...TrieRest) as GhtForest<SearchKey>>::find_and_force(
                rest, search_key,
            )
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
    fn find_and_force<'a>(&mut self, _search_key: SearchKey) -> bool {
        false
    }
}

#[sealed]
impl<SearchKey> GhtForest<SearchKey> for var_type!()
where
    SearchKey: VariadicExt,
{
    fn find_and_force<'a>(&mut self, _search_key: SearchKey) -> bool {
        false
    }
}

#[sealed]
/// a trait for finding a matching leaf in the forest
pub trait ForestFindLeaf<Schema, Storage>
where
    Schema: Eq + Hash + VariadicExt + PartialEqVariadic,
    Storage: VariadicCollection<Schema = Schema>,
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
    Storage: VariadicCollection<Schema = Schema>,
{
    fn find_containing_leaf(
        &self,
        _row: <Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<Schema, (), Storage>> {
        None
    }
}
