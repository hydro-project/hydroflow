use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use sealed::sealed;
use variadics::{
    var_args, var_type, PartialEqVariadic, RefVariadic, Split, SplitBySuffix, VariadicExt,
};

use crate::ght_lattice::DeepJoinLatticeBimorphism;

/// GeneralizedHashTrieNode trait
#[sealed]
pub trait GeneralizedHashTrieNode: Default {
    /// Schema variadic: the schema of the relation stored in this trie.
    /// This type is the same in all nodes of the trie.
    type Schema: VariadicExt + Eq + Hash + Clone + SplitBySuffix<Self::SuffixSchema>;
    /// The prefix of columns in Schema that the trie is keyed on
    /// This type is the same in all nodes of the trie.
    type KeyType: VariadicExt + Eq + Hash + Clone;
    /// The suffix of columns in Schema that are not part of the trie keys
    /// This type is the same in all nodes of the trie.
    type ValType: VariadicExt + Eq + Hash + Clone;

    /// SuffixSchema variadic: the suffix of the schema *from this node of the trie
    /// downward*. The first entry in this variadic is of type Head.
    type SuffixSchema: VariadicExt + Eq + Hash + Clone;
    /// The type of the first column in the SuffixSchema
    type Head: Eq + Hash;

    /// Create a new Ght from the iterator.
    fn new_from(input: impl IntoIterator<Item = Self::Schema>) -> Self;

    /// Report the height of the tree if its not empty. This is the length of a root to leaf path -1.
    /// E.g. if we have GhtInner<GhtInner<GhtLeaf...>> the height is 2
    fn height(&self) -> Option<usize>;

    /// debugging function to check that height is consistent along at least the leftmost path
    fn check_height(&self) -> bool;

    /// report whether node is a leaf node; else an inner node
    fn is_leaf(&self) -> bool;

    /// Inserts items into the hash trie. Returns the height
    /// of this node (leaf = 0)
    fn insert(&mut self, row: Self::Schema) -> Option<usize>;

    /// Returns `true` if the (entire) row is found in the trie, `false` otherwise.
    /// See `get()` below to look just for "head" keys in this node
    fn contains<'a>(&'a self, row: <Self::Schema as VariadicExt>::AsRefVar<'a>) -> bool;

    /// Iterator for the "head" keys (from inner nodes) or elements (from leaf nodes).
    fn iter(&self) -> impl Iterator<Item = &'_ Self::Head>;

    /// Type returned by [`Self::get`].
    type Get: GeneralizedHashTrieNode<Schema = Self::Schema>;

    /// On an Inner node, retrieves the value (child) associated with the given "head" key.
    /// returns an `Option` containing a reference to the value if found, or `None` if not found.
    /// On a Leaf node, returns None.
    fn get(&self, head: &Self::Head) -> Option<&'_ Self::Get>;

    /// Iterate through the (entire) rows stored in this HashTrie.
    /// Returns Variadics, not tuples.
    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>>;

    // /// Iterate through the (entire) rows stored in this HashTrie, but with the leaf
    // /// values stubbed out ((), ()).
    // fn recursive_iter_keys(
    //     &self,
    // ) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>>;

    /// Bimorphism for joining on full tuple keys (all GhtInner keys) in the trie
    type DeepJoin<Other>
    where
        Other: GeneralizedHashTrieNode,
        (Self, Other): DeepJoinLatticeBimorphism;

    // /// For Inner nodes only, this is the type of the Child node
    // type ChildNode: GeneralizedHashTrieNode;

    // /// Cast as Some<&GhtInner> if this is an inner node, else return None
    // fn cast_as_inner(&self) -> Option<&Self>;

    // /// Cast as Some<&mut GhtInner> if this is an inner node, else return None
    // fn cast_as_inner_mut(&mut self) -> Option<&mut Self>;

    /// return the leaf in the trie that contains this row
    fn find_containing_leaf(
        &self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<Self::Schema, Self::ValType>>;
}

/// A trait for internal nodes of a GHT
pub trait GhtHasChildren: GeneralizedHashTrieNode {
    /// The child node's type
    type Node: GeneralizedHashTrieNode;
    /// return the hash map of children, mutable
    fn children(&mut self) -> &mut HashMap<Self::Head, Self::Node>;
}

/// internal node of a HashTrie
#[derive(Debug, Clone)]
pub struct GhtInner<Head, Node>
where
    Head: Clone,
    Node: GeneralizedHashTrieNode,
{
    pub(crate) children: HashMap<Head, Node>,
    pub(crate) height: Option<usize>,
    // pub(crate) _leaf: std::marker::PhantomData<Leaf>,
}
impl<Head, Node: GeneralizedHashTrieNode> Default for GhtInner<Head, Node>
where
    Head: Clone,
    Node: GeneralizedHashTrieNode,
{
    fn default() -> Self {
        let children = Default::default();
        Self {
            children,
            height: None,
            // _leaf: Default::default(),
        }
    }
}
#[sealed]
impl<Head, Node> GeneralizedHashTrieNode for GhtInner<Head, Node>
where
    Head: 'static + Hash + Eq + Clone,
    Node: 'static + GeneralizedHashTrieNode,
    Node::Schema: SplitBySuffix<var_type!(Head, ...Node::SuffixSchema)>,
{
    type Schema = Node::Schema;
    type SuffixSchema = var_type!(Head, ...Node::SuffixSchema);
    type ValType = Node::ValType;
    type KeyType = Node::KeyType;

    type Head = Head;

    fn new_from(input: impl IntoIterator<Item = Self::Schema>) -> Self {
        let mut retval: Self = Default::default();
        let mut height = None;
        for row in input {
            // let (_prefix, suffix) = row.clone().split();
            height = retval.insert(row);
        }
        retval.height = height;
        retval
    }

    fn height(&self) -> Option<usize> {
        self.height
    }

    fn check_height(&self) -> bool {
        self.height.unwrap()
            == self
                .children
                .iter()
                .next()
                .and_then(|n| n.1.height())
                .map(|h| h + 1)
                .unwrap()
    }

    fn is_leaf(&self) -> bool {
        false
    }

    fn insert(&mut self, row: Self::Schema) -> Option<usize> {
        // TODO(mingwei): clones entire row...
        let (_prefix, var_args!(head, ..._rest)) = row.clone().split_by_suffix();
        self.height = self
            .children
            .entry(head)
            .or_default()
            .insert(row)
            .map(|h| h + 1);
        self.height
    }

    fn contains<'a>(&'a self, row: <Self::Schema as VariadicExt>::AsRefVar<'a>) -> bool {
        //  as SplitBySuffix<Self::SuffixSchema>>
        let (_prefix, var_args!(head, ..._rest)) = Self::Schema::split_by_suffix_ref(row);
        if let Some(node) = self.children.get(head) {
            node.contains(row)
        } else {
            false
        }
    }

    fn iter(&self) -> impl Iterator<Item = &'_ Self::Head> {
        self.children.keys()
    }

    type Get = Node;
    fn get(&self, head: &Self::Head) -> Option<&'_ Self::Get> {
        self.children.get(head)
    }

    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
        self.children
            .iter()
            .flat_map(|(_k, vs)| vs.recursive_iter())
    }

    // fn recursive_iter_keys(
    //     &self,
    // ) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
    //     self.children
    //         .iter()
    //         .flat_map(|(_k, vs)| vs.recursive_iter_keys().map(move |v| v))
    // }

    type DeepJoin<Other> = <(Self, Other) as DeepJoinLatticeBimorphism>::DeepJoinLatticeBimorphism
    where
        Other: GeneralizedHashTrieNode,
        (Self, Other): DeepJoinLatticeBimorphism;

    fn find_containing_leaf(
        &self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<Self::Schema, Self::ValType>> {
        let (_prefix, var_args!(head, ..._rest)) = Self::Schema::split_by_suffix_ref(row);
        self.children
            .get(head)
            .and_then(|child| child.find_containing_leaf(row))
    }
}
impl<Head, Node> FromIterator<Node::Schema> for GhtInner<Head, Node>
where
    Head: 'static + Hash + Eq + Clone,
    Node: 'static + GeneralizedHashTrieNode + Clone,
    Node::Schema: SplitBySuffix<var_type!(Head, ...Node::SuffixSchema)>,
{
    fn from_iter<Iter: IntoIterator<Item = Node::Schema>>(iter: Iter) -> Self {
        let mut out = Self::default();
        for row in iter {
            out.insert(row);
        }
        out
    }
}

impl<Schema, ValType> FromIterator<Schema> for GhtLeaf<Schema, ValType>
where
    Schema: Eq + Hash,
{
    fn from_iter<Iter: IntoIterator<Item = Schema>>(iter: Iter) -> Self {
        let elements = iter.into_iter().collect();
        Self {
            elements,
            _suffix_schema: PhantomData,
        }
    }
}

impl<Head, Node> GhtHasChildren for GhtInner<Head, Node>
where
    Head: 'static + Hash + Eq + Clone,
    Node: 'static + GeneralizedHashTrieNode + Clone,
    Node::Schema: SplitBySuffix<var_type!(Head, ...Node::SuffixSchema)>,
{
    type Node = Node;
    fn children(&mut self) -> &mut HashMap<Head, Node> {
        &mut self.children
    }
}

/// leaf node of a HashTrie
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(transparent)]
pub struct GhtLeaf<Schema, ValType>
where
    Schema: Eq + Hash,
{
    pub(crate) elements: HashSet<Schema>,
    pub(crate) _suffix_schema: PhantomData<ValType>,
}
impl<Schema, ValType> Default for GhtLeaf<Schema, ValType>
where
    Schema: Eq + Hash,
{
    fn default() -> Self {
        let elements = Default::default();
        Self {
            elements,
            _suffix_schema: PhantomData,
        }
    }
}
#[sealed]
impl<Schema, ValType> GeneralizedHashTrieNode for GhtLeaf<Schema, ValType>
where
    Schema: 'static + Eq + VariadicExt + Hash + Clone + SplitBySuffix<ValType> + PartialEqVariadic,
    ValType: VariadicExt + Clone + Eq + Hash + PartialEqVariadic,
    <Schema as SplitBySuffix<ValType>>::Prefix: Eq + Hash + Clone,
    // for<'a> Schema::AsRefVar<'a>: PartialEq,
{
    type Schema = Schema;
    type SuffixSchema = ValType;
    type ValType = ValType;
    type KeyType = <Schema as SplitBySuffix<ValType>>::Prefix;
    type Head = Schema;

    fn new_from(input: impl IntoIterator<Item = Self::Schema>) -> Self {
        let mut retval: Self = Default::default();
        for i in input {
            retval.insert(i);
        }
        retval
    }

    fn height(&self) -> Option<usize> {
        Some(0)
    }

    fn check_height(&self) -> bool {
        true
    }

    fn is_leaf(&self) -> bool {
        true
    }

    fn insert(&mut self, row: Self::Schema) -> Option<usize> {
        self.elements.insert(row);
        self.height()
    }

    fn contains<'a>(&'a self, row: <Self::Schema as VariadicExt>::AsRefVar<'a>) -> bool {
        self.elements
            .iter()
            .any(|r| Schema::eq_ref(r.as_ref_var(), row))
    }

    fn iter(&self) -> impl Iterator<Item = &'_ Self::Head> {
        self.elements.iter()
    }

    type Get = Self;
    fn get(&self, _head: &Self::Head) -> Option<&'_ Self::Get> {
        Option::<&Self>::None
    }

    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
        self.elements.iter().map(Schema::as_ref_var)
    }

    // fn recursive_iter_keys(
    //     &self,
    // ) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
    //     let out = self.elements.iter().map(Schema::as_ref_var).next().unwrap();
    //     std::iter::once(out)
    // }

    type DeepJoin<Other> = <(Self, Other) as DeepJoinLatticeBimorphism>::DeepJoinLatticeBimorphism
    where
        Other: GeneralizedHashTrieNode,
        (Self, Other): DeepJoinLatticeBimorphism;

    fn find_containing_leaf(
        &self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<<Self as GeneralizedHashTrieNode>::Schema, Self::ValType>> {
        // TODO(mingwei): actually use the hash set as a hash set
        if self.elements.iter().any(|x| {
            // let (_prefix, suffix) = Schema::split_by_suffix_ref(x.as_ref_var());
            <Schema as PartialEqVariadic>::eq_ref(row, x.as_ref_var())
        }) {
            Some(self)
        } else {
            None
        }
    }
}

// #[sealed]
// /// iterators for HashTries based on a prefix search
// pub trait FindLeaf: GeneralizedHashTrieNode {
//     /// given a prefix, return an iterator through the items below
//     fn find_containing_leaf(
//         &self,
//         row: <Self::SuffixSchema as VariadicExt>::AsRefVar<'_>,
//     ) -> Option<&'_ GhtLeaf<Self::Schema, Self::ValType>>;
// }

// // #[sealed]
// // impl<KeyType, ValType, TrieRoot, Schema> FindLeaf<Schema> for GHT<KeyType, ValType, TrieRoot>
// // where
// //     TrieRoot: FindLeaf<Schema>,
// //     KeyType: VariadicExt,
// //     ValType: 'static + VariadicExt + Eq + Hash + PartialEqVariadic + Clone,
// //     TrieRoot: GeneralizedHashTrieNode,
// //     Schema: Eq + Hash + VariadicExt + PartialEqVariadic,
// //     for<'a> ValType::AsRefVar<'a>: PartialEq,
// //     // ValType: SplitBySuffix<SuffixSchema>,
// //     // SuffixSchema: Clone + VariadicExt,
// // {
// //     type Schema = Schema;

// //     // type Suffix = <TrieRoot as HtPrefixIter<KeyPrefix>>::Suffix;
// //     type Suffix = <TrieRoot as FindLeaf<Schema>>::Suffix;

// //     fn find_containing_leaf(&self, row: Schema) -> Option<&'_ GhtLeaf<Schema, ValType>> {
// //         self.trie.find_containing_leaf(row)
// //     }
// // }

// // var_type!(&'k Head, ...SchemaRest)
// // var_type!(&'k Head, ...Node::SuffixSchema)
// #[sealed]
// impl<'k, Head, Node> FindLeaf for GhtInner<Head, Node>
// where
//     Self: GeneralizedHashTrieNode,
//     Head: Eq + Hash + Clone,
//     Node: FindLeaf<
//         Schema = <Self as GeneralizedHashTrieNode>::Schema,
//         ValType = <Self as GeneralizedHashTrieNode>::ValType,
//     >,
// {
//     fn find_containing_leaf(
//         &self,
//         row: <Self::SuffixSchema as VariadicExt>::AsRefVar<'_>,
//     ) -> Option<&'_ GhtLeaf<Self::Schema, Self::ValType>> {
//         let var_args!(head, ...rest) = row;
//         self.children
//             .get(head)
//             .and_then(|child| child.find_containing_leaf(rest))
//     }
// }

// // #[sealed]
// // impl<'k, Head, PrefixRest> HtPrefixIter<var_type!(&'k Head, ...PrefixRest::AsRefVar<'k>)>
// //     for GhtLeaf<var_type!(Head, ...PrefixRest)>
// // where
// //     Head: Eq + Hash,
// //     PrefixRest: Eq + Hash + VariadicExt + PartialEqVariadic,

// /// This case only splits HEAD and REST in order to prevent a conflict with the `HtPrefixIter<var_type!()>` impl.
// /// If not for that, we could just use a single variadic type parameter.
// #[sealed]
// impl<'k, Schema, ValType> FindLeaf for GhtLeaf<Schema, ValType>
// where
//     Self: GeneralizedHashTrieNode<Schema = Schema, ValType = ValType, SuffixSchema = ValType>,
//     Schema: Eq + Hash + SplitBySuffix<ValType>,
//     ValType: Eq + Hash + PartialEqVariadic,
//     // Head: Eq + Hash + RefVariadic, /* TODO(mingwei): `Hash` actually use hash set contains instead of iterate. */
//     // Head::UnRefVar: 'static + Eq + Hash,
//     // for<'a> PrefixRest::AsRefVar<'a>: PartialEq<PrefixRest::AsRefVar<'a>>,
//     // Head::UnRefVar: for<'a> VariadicExt<AsRefVar<'a> = Head>,
// {
//     fn find_containing_leaf(
//         &self,
//         row: <Self::SuffixSchema as VariadicExt>::AsRefVar<'_>,
//     ) -> Option<&'_ GhtLeaf<<Self as GeneralizedHashTrieNode>::Schema, Self::ValType>> {
//         // TODO(mingwei): actually use the hash set as a hash set
//         if self.elements.iter().any(|x| {
//             let (_prefix, suffix) = Schema::split_by_suffix_ref(x.as_ref_var());
//             <ValType as PartialEqVariadic>::eq_ref(row, suffix)
//         }) {
//             Some(self)
//         } else {
//             None
//         }
//         // let var_args!(head) = prefix;
//         // self.elements.contains(head).then_some(()).into_iter()
//     }
// }

/// Trait for taking a matching leaf out of a trie, as part of the COLT force
#[sealed]
pub trait GhtTakeLeaf: GeneralizedHashTrieNode {
    /// take a matching leaf out of a trie, as part of the COLT force
    fn take_containing_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<GhtLeaf<Self::Schema, Self::ValType>>;

    /// Merges a leaf into the hash trie
    fn merge_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
        leaf: GhtLeaf<Self::Schema, Self::ValType>,
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
        leaf: GhtLeaf<Self::Schema, Self::ValType>,
    ) -> bool {
        // TODO(mingwei): clones head...
        let (_prefix, var_args!(head, ..._rest)) = Self::Schema::split_by_suffix_ref(row);
        let retval = self
            .children
            .entry(head.clone())
            .or_default()
            .merge_leaf(row, leaf);
        self.height = self.get(head).and_then(|n| n.height).map(|h| h + 1);
        retval
    }
}

#[sealed]
impl<Head, Schema, ValType> GhtTakeLeaf for GhtInner<Head, GhtLeaf<Schema, ValType>>
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
    <Self as GeneralizedHashTrieNode>::Schema:
        SplitBySuffix<<Self as GeneralizedHashTrieNode>::SuffixSchema>,
{
    fn take_containing_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<GhtLeaf<Self::Schema, Self::ValType>> {
        let (_prefix, var_args!(head, ..._rest)) =
            <<Self as GeneralizedHashTrieNode>::Schema as SplitBySuffix<
                <Self as GeneralizedHashTrieNode>::SuffixSchema,
            >>::split_by_suffix_ref(row);
        self.children.remove(head)
    }

    fn merge_leaf(
        &mut self,
        row: <Self::Schema as VariadicExt>::AsRefVar<'_>,
        leaf: GhtLeaf<Self::Schema, Self::ValType>,
    ) -> bool {
        let (_prefix, var_args!(head, ..._rest)) =
            <<Self as GeneralizedHashTrieNode>::Schema as SplitBySuffix<
                <Self as GeneralizedHashTrieNode>::SuffixSchema,
            >>::split_by_suffix_ref(row);
        if let Some(old_val) = self.children.insert(head.clone(), leaf) {
            self.children.insert(head.clone(), old_val);
            panic!();
            false
        } else {
            self.height = Some(1);
            true
        }
    }
}

#[sealed]
/// iterators for HashTries based on a prefix search
pub trait GhtPrefixIter<KeyPrefix> {
    /// the schema output
    type Item: VariadicExt;
    /// given a prefix, return an iterator through the items below
    fn prefix_iter<'a>(
        &'a self,
        prefix: KeyPrefix,
    ) -> impl Iterator<Item = <Self::Item as VariadicExt>::AsRefVar<'a>>
    where
        Self::Item: 'a;
}

#[sealed]
impl<'k, Head, Node, PrefixRest> GhtPrefixIter<var_type!(&'k Head, ...PrefixRest)>
    for GhtInner<Head, Node>
where
    Head: Eq + Hash + Clone,
    Node: GeneralizedHashTrieNode + GhtPrefixIter<PrefixRest>,
{
    type Item = <Node as GhtPrefixIter<PrefixRest>>::Item;
    fn prefix_iter<'a>(
        &'a self,
        prefix: var_type!(&'k Head, ...PrefixRest),
    ) -> impl Iterator<Item = <Self::Item as VariadicExt>::AsRefVar<'a>>
    where
        Self::Item: 'a,
    {
        let var_args!(head, ...rest) = prefix;
        self.children
            .get(head)
            .map(|node| node.prefix_iter(rest))
            .into_iter()
            .flatten()
    }
}
#[sealed]
impl<Head, Node> GhtPrefixIter<var_type!()> for GhtInner<Head, Node>
where
    Self: GeneralizedHashTrieNode,
    Head: Eq + Hash + Clone,
    Node: GeneralizedHashTrieNode,
{
    type Item = <Self as GeneralizedHashTrieNode>::Schema;
    fn prefix_iter<'a>(
        &'a self,
        _prefix: var_type!(),
    ) -> impl Iterator<Item = <Self::Item as VariadicExt>::AsRefVar<'a>>
    where
        Self::Item: 'a,
    {
        self.recursive_iter()
    }
}

/// This case only splits HEAD and REST in order to prevent a conflict with the `HtPrefixIter<var_type!()>` impl.
/// If not for that, we could just use a single variadic type parameter.
#[sealed]
impl<KeyPrefixRef, Schema, ValType> GhtPrefixIter<KeyPrefixRef> for GhtLeaf<Schema, ValType>
where
    KeyPrefixRef: 'static + RefVariadic,
    Schema: 'static + VariadicExt + Hash + Eq + SplitBySuffix<ValType>,
    ValType: VariadicExt,
    // for<'a> SuffixSchema::AsRefVar<'a>: Split<KeyPrefixRef>,
    ValType: Split<KeyPrefixRef::UnRefVar>,
    KeyPrefixRef::UnRefVar: PartialEqVariadic,
{
    type Item = Schema;
    fn prefix_iter<'a>(
        &'a self,
        prefix: KeyPrefixRef,
    ) -> impl Iterator<Item = <Self::Item as VariadicExt>::AsRefVar<'a>>
    where
        Self::Item: 'a,
    {
        self.elements
            .iter()
            .map(Schema::as_ref_var)
            .filter(move |&row| {
                let (_row_prefix, row_mid_suffix) =
                    <Schema as SplitBySuffix<ValType>>::split_by_suffix_ref(row);
                let (row_mid, _row_suffix): (
                    <KeyPrefixRef::UnRefVar as VariadicExt>::AsRefVar<'_>,
                    _,
                ) = <ValType as Split<KeyPrefixRef::UnRefVar>>::split_ref(row_mid_suffix);
                <KeyPrefixRef::UnRefVar as PartialEqVariadic>::eq_ref(prefix.unref_ref(), row_mid)
            })
    }
}

// #[sealed]
// impl<Head, Node> HtPrefixIter<var_type!()> for GhtInner<Head, Node>
// where
//     Head: 'static + Eq + Hash,
//     Node: 'static + GeneralizedHashTrieNode,
// {
//     type Suffix = <Self as GeneralizedHashTrieNode>::Schema;
//     fn prefix_iter<'a>(
//         &'a self,
//         _prefix: var_type!(),
//     ) -> impl Iterator<Item = <Self::Suffix as VariadicExt>::AsRefVar<'a>>
//     where
//         Self::Suffix: 'a,
//     {
//         self.recursive_iter()
//     }
// }

// // #[sealed]
// // impl<'k, Head, PrefixRest> HtPrefixIter<var_type!(&'k Head, ...PrefixRest::AsRefVar<'k>)>
// //     for GhtLeaf<var_type!(Head, ...PrefixRest)>
// // where
// //     Head: Eq + Hash,
// //     PrefixRest: Eq + Hash + VariadicExt + PartialEqVariadic,
// //     // Head: Eq + Hash + RefVariadic, /* TODO(mingwei): `Hash` actually use hash set contains instead of iterate. */
// //     // Head::UnRefVar: 'static + Eq + Hash,
// //     // for<'a> PrefixRest::AsRefVar<'a>: PartialEq<PrefixRest::AsRefVar<'a>>,
// //     // Head::UnRefVar: for<'a> VariadicExt<AsRefVar<'a> = Head>,
// // {
// //     type Suffix = var_expr!();
// //     fn prefix_iter<'a>(
// //         &'a self,
// //         prefix: var_type!(&'k Head, ...PrefixRest::AsRefVar<'k>),
// //     ) -> impl Iterator<Item = <Self::Suffix as VariadicExt>::AsRefVar<'a>>
// //     where
// //         Self::Suffix: 'a,
// //     {
// //         // TODO(mingwei): actually use the hash set as a hash set
// //         let var_args!(prefix_head, ...prefix_rest) = prefix;
// //         self.elements
// //             .iter()
// //             .any(|var_args!(item_head, ...item_rest)| {
// //                 prefix_head == item_head && PrefixRest::eq_ref(prefix_rest, item_rest.as_ref_var())
// //             })
// //             .then_some(())
// //             .into_iter()
// //         // let var_args!(head) = prefix;
// //         // self.elements.contains(head).then_some(()).into_iter()
// //     }
// // }
// #[sealed]
// impl<This> HtPrefixIter<var_type!()> for This
// where
//     This: 'static + GeneralizedHashTrieNode,
// {
//     type Schema = <Self as GeneralizedHashTrieNode>::Schema;
//     fn prefix_iter<'a>(
//         &'a self,
//         _prefix: var_type!(),
//     ) -> impl Iterator<Item = <This::Schema as VariadicExt>::AsRefVar<'a>>
//     where
//         Self::Schema: 'a,
//     {
//         self.recursive_iter()
//     }
// }

// trait ContainsKey {
//     fn contains_key(&self) -> bool;
// }

// impl<Head, Node> ContainsKey for GhtInner<Head, Node>
// where
//     Head: 'static + Hash + Eq,
//     Node: 'static + GeneralizedHashTrieNode,
// {
//     /// Returns `true` if the key is found in the inner nodes of the trie, `false` otherwise.
//     fn contains_key(&self, key: <Self::KeyType as VariadicExt>::AsRefVar<'_>) -> bool {
//         true
//     }
// }

/// Helper that does the heavy lifting for GhtNodeType!
#[macro_export]
macro_rules! GhtNodeTypeWithSchema {
    // Empty key & Val (Leaf)
    (() => () => $( $schema:ty ),+ ) => (
        $crate::ght::GhtLeaf::<$( $schema ),*,  ()  >
    );

    // Empty key (Leaf)
    (() => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtLeaf::<$schema,  $crate::variadics::var_type!($( $z ),* ) >
    );

    // Singleton key & Empty val (Inner over Leaf)
    ($a:ty => () => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$schema, () >>
    );

    // Singleton key (Inner over Leaf)
    ($a:ty => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$schema, $crate::variadics::var_type!($( $z ),*) >>
    );

    // Recursive case with empty val
    ($a:ty, $( $b:ty ),* => () => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::GhtNodeTypeWithSchema!($( $b ),* => () => $schema)>
    );

    // Recursive case.
    ($a:ty, $( $b:ty ),* => $( $z:ty ),* => $schema:ty ) => (
        $crate::ght::GhtInner::<$a, $crate::GhtNodeTypeWithSchema!($( $b ),* => $( $z ),* => $schema)>
    );
}

/// Macro to construct a Ght node type from the constituent key and
/// dependent column types. You pass it:
///    - a list of key column types and dependent column type separated by a fat arrow,
///         a la (K1, K2, K3 => T1, T2, T3)
///
/// This macro generates a hierarchy of GHT node types where each key column is associated with an GhtInner
/// of the associated column type, and the remaining dependent columns are associated with a variadic HTleaf
/// a la var_expr!(T1, T2, T3)
#[macro_export]
macro_rules! GhtType {
    // Empty key
    (() => $( $z:ty ),* ) => (
        $crate::GhtNodeTypeWithSchema!(() => $( $z ),* => $crate::variadics::var_type!($( $z ),* ))
    );

    // Recursive case empty val
    ($( $b:ty ),* => () ) => (
        $crate::GhtNodeTypeWithSchema!($( $b ),* => () => $crate::variadics::var_type!($( $b ),*))
    );

    // Recursive case
    ($( $b:ty ),* => $( $z:ty ),*) => (
        $crate::GhtNodeTypeWithSchema!($( $b ),* => $( $z ),* => $crate::variadics::var_type!($( $b ),*, $( $z ),*))
    );
}
