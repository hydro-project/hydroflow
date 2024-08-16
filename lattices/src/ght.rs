use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use sealed::sealed;
use variadics::{
    var_args, var_type, PartialEqVariadic, RefVariadic, Split, SplitBySuffix, VariadicExt,
};

use crate::ght_lattice::DeepJoinLatticeBimorphism;

// TODO(mingwei): GET RID OF THIS TRAIT, MERGE INTO IMPL FOR GHT.
/// GeneralizedHashTrie wraps up a root GeneralizedHashTrieNode with metadata
/// for the key and value types associated with the full trie.
pub trait GeneralizedHashTrie {
    /// the prefix of the Schema representing the Key type
    type KeyType: VariadicExt;
    type Trie: GeneralizedHashTrieNode<
        SuffixSchema = <Self::Trie as GeneralizedHashTrieNode>::Schema,
    >;

    /// Create a new Ght from the iterator.
    fn new_from(
        input: impl IntoIterator<Item = <Self::Trie as GeneralizedHashTrieNode>::Schema>,
    ) -> Self;

    /// Report the height of the tree if its not empty. This is the length of a root to leaf path -1.
    /// E.g. if we have GhtInner<GhtInner<GhtLeaf...>> the height is 2
    fn height(&self) -> Option<usize>;

    /// Inserts items into the hash trie.
    fn insert(&mut self, row: <Self::Trie as GeneralizedHashTrieNode>::Schema) -> bool;

    /// Returns `true` if the (entire) row is found in the trie, `false` otherwise.
    fn contains<'a>(
        &'a self,
        row: <<Self::Trie as GeneralizedHashTrieNode>::Schema as VariadicExt>::AsRefVar<'a>,
    ) -> bool;

    // /// walk to the leaf from any inner node
    // fn walk_to_leaf<Node, Head, Rest>(
    //     node: Node,
    //     search_key: var_type!(Head, ...Rest),
    // ) -> Option<GhtLeaf<Self::ValType>>
    // where
    //     Node: GeneralizedHashTrieNode + GhtHasChildren,
    //     Rest: VariadicExt;

    /// Iterate through the (entire) rows stored in this HashTrie.
    /// Returns Variadics, not tuples.
    fn recursive_iter(
        &self,
    ) -> impl Iterator<
        Item = <<Self::Trie as GeneralizedHashTrieNode>::Schema as VariadicExt>::AsRefVar<'_>,
    >;

    /// Iterate through the (entire) rows stored in this HashTrie, but with the leaf
    /// values stubbed out ((), ()).
    // fn recursive_iter_keys(
    //     &self,
    // ) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>>;

    // /// Extract Key and Value from a by-value tuple
    // fn split_key_val_no_refs(tup: Self::Schema) -> (Self::KeyType, Self::ValType)
    // where
    //     Self::Schema: Split<Self::KeyType, Suffix = Self::ValType>,
    // {
    //     tup.split()
    // }

    /// Extract Key and Value from a returned tuple
    // fn split_key_val<'a>(
    //     tup: <Self::Schema as VariadicExt>::AsRefVar<'a>,
    // ) -> (
    //     <Self::KeyType as VariadicExt>::AsRefVar<'a>,
    //     <Self::ValType as VariadicExt>::AsRefVar<'a>,
    // )
    // where
    //     <Self::Schema as VariadicExt>::AsRefVar<'a>: Split<
    //         <Self::KeyType as VariadicExt>::AsRefVar<'a>,
    //         Suffix = <Self::ValType as VariadicExt>::AsRefVar<'a>,
    //     >,
    // {
    //     tup.split()
    // }

    /// get a ref to the underlying root note of the trie
    fn get_trie(&self) -> &Self::Trie;

    /// get a mutable ref to the underlying root note of the trie
    fn get_mut_trie(&mut self) -> &mut Self::Trie;

    fn find_containing_leaf(
        &self,
        row: <<Self::Trie as GeneralizedHashTrieNode>::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<
        &'_ GhtLeaf<
            <Self::Trie as GeneralizedHashTrieNode>::Schema,
            <Self::Trie as GeneralizedHashTrieNode>::ValType,
        >,
    >;
}

/// GeneralizedHashTrie is a metadata node pointing to a root GeneralizedHashTrieNode.
#[derive(Debug, Clone)]
pub struct GHT<KeyType, TrieRoot>
where
    KeyType: VariadicExt, // + AsRefVariadicPartialEq
    TrieRoot: GeneralizedHashTrieNode,
{
    pub(crate) trie: TrieRoot,
    pub(crate) _key: PhantomData<KeyType>,
}

impl<K, TrieRoot> GHT<K, TrieRoot>
where
    K: VariadicExt, // + AsRefVariadicPartialEq
    TrieRoot: GeneralizedHashTrieNode<SuffixSchema = <TrieRoot as GeneralizedHashTrieNode>::Schema>,
{
    /// Just calls `prefix_iter` on the underlying trie.
    pub fn prefix_iter<'a, KeyPrefix>(
        &'a self,
        prefix: KeyPrefix,
    ) -> impl Iterator<Item = <<TrieRoot as HtPrefixIter<KeyPrefix>>::Item as VariadicExt>::AsRefVar<'a>>
    where
        TrieRoot: HtPrefixIter<KeyPrefix>,
        KeyPrefix: 'a,
    {
        self.trie.prefix_iter(prefix)
    }
}

impl<K, TrieRoot, Schema> GeneralizedHashTrie for GHT<K, TrieRoot>
where
    K: VariadicExt, // + AsRefVariadicPartialEq
    Schema: VariadicExt + Eq + Hash,
    TrieRoot: GeneralizedHashTrieNode<Schema = Schema, SuffixSchema = Schema>,
{
    type KeyType = K;
    type Trie = TrieRoot;

    fn new_from(
        input: impl IntoIterator<Item = <Self::Trie as GeneralizedHashTrieNode>::Schema>,
    ) -> Self {
        let trie = GeneralizedHashTrieNode::new_from(input);
        GHT {
            trie,
            _key: Default::default(),
        }
    }

    fn height(&self) -> Option<usize> {
        self.trie.height()
    }

    fn insert(&mut self, row: <Self::Trie as GeneralizedHashTrieNode>::Schema) -> bool {
        self.trie.insert(row)
    }

    fn contains<'a>(
        &'a self,
        row: <<Self::Trie as GeneralizedHashTrieNode>::Schema as VariadicExt>::AsRefVar<'a>,
    ) -> bool {
        self.trie.contains(row)
    }

    /// Iterate through the (entire) rows stored in this HashTrie.
    /// Returns Variadics, not tuples.
    fn recursive_iter(
        &self,
    ) -> impl Iterator<
        Item = <<Self::Trie as GeneralizedHashTrieNode>::Schema as VariadicExt>::AsRefVar<'_>,
    > {
        self.trie.recursive_iter()
    }

    /// Iterate through the (entire) rows stored in this HashTrie, but with the leaf
    /// values stubbed out ((), ()).
    // fn recursive_iter_keys(
    //     &self,
    // ) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
    //     self.trie.recursive_iter_keys()
    // }

    fn get_trie(&self) -> &TrieRoot {
        &self.trie
    }

    /// get a mutable ref to the underlying root note of the trie
    fn get_mut_trie(&mut self) -> &mut Self::Trie {
        &mut self.trie
    }

    fn find_containing_leaf(
        &self,
        row: <<Self::Trie as GeneralizedHashTrieNode>::Schema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<
        &'_ GhtLeaf<
            <Self::Trie as GeneralizedHashTrieNode>::Schema,
            <Self::Trie as GeneralizedHashTrieNode>::ValType,
        >,
    > {
        self.trie.find_containing_leaf(row)
    }
}

impl<K, TrieRoot> Default for GHT<K, TrieRoot>
where
    K: VariadicExt, // + AsRefVariadicPartialEq
    TrieRoot: GeneralizedHashTrieNode<SuffixSchema = <TrieRoot as GeneralizedHashTrieNode>::Schema>,
{
    fn default() -> Self {
        let tree = TrieRoot::default();
        let _key: PhantomData<K> = Default::default();
        Self { trie: tree, _key }
    }
}

/// GeneralizedHashTrieNode trait
#[sealed]
pub trait GeneralizedHashTrieNode: Default {
    /// Schema variadic: the schema of the relation stored in this trie.
    /// This type is the same in all nodes of the trie.
    type Schema: VariadicExt + Eq + Hash + Clone + SplitBySuffix<Self::SuffixSchema>;

    /// SuffixSchema variadic: the suffix of the schema from this node of the trie
    /// downward. First entry in the variadic is of type Head
    type SuffixSchema: VariadicExt + Eq + Hash + Clone;

    type ValType: VariadicExt + Eq + Hash + Clone;

    /// The type of the first column in the SuffixSchema
    type Head: Eq + Hash;

    /// The type of the leaves
    // type Leaf: Eq + Hash;

    /// Create a new Ght from the iterator.
    fn new_from(input: impl IntoIterator<Item = Self::Schema>) -> Self;

    /// Report the height of the tree if its not empty. This is the length of a root to leaf path -1.
    /// E.g. if we have GhtInner<GhtInner<GhtLeaf...>> the height is 2
    fn height(&self) -> Option<usize>;

    /// report whether node is a leaf node; else an inner node
    fn is_leaf(&self) -> bool;

    /// Inserts items into the hash trie.
    fn insert(&mut self, row: Self::Schema) -> bool;

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

    fn find_containing_leaf(
        &self,
        row: <Self::SuffixSchema as VariadicExt>::AsRefVar<'_>,
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

    type Head = Head;

    fn new_from(input: impl IntoIterator<Item = Self::Schema>) -> Self {
        let mut retval: Self = Default::default();
        for row in input {
            // let (_prefix, suffix) = row.clone().split();
            retval.insert(row);
        }
        retval
    }

    fn height(&self) -> Option<usize> {
        if let Some((_k, v)) = self.children.iter().next() {
            Some(v.height().unwrap() + 1)
        } else {
            None
        }
    }

    fn is_leaf(&self) -> bool {
        false
    }

    fn insert(&mut self, row: Self::Schema) -> bool {
        // TODO(mingwei): clones entire row...
        let (_prefix, var_args!(head, ..._rest)) = row.clone().split_by_suffix();
        self.children.entry(head).or_default().insert(row)
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

    // type ChildNode = Node;
    // fn cast_as_inner(&self) -> Option<&Self> {
    //     Some(self)
    // }
    // fn cast_as_inner_mut(&mut self) -> Option<&mut Self> {
    //     Some(self)
    // }

    fn find_containing_leaf(
        &self,
        row: <Self::SuffixSchema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<Self::Schema, Self::ValType>> {
        let var_args!(head, ...rest) = row;
        self.children
            .get(head)
            .and_then(|child| child.find_containing_leaf(rest))
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
    // for<'a> Schema::AsRefVar<'a>: PartialEq,
{
    type Schema = Schema;
    type SuffixSchema = ValType;
    type ValType = ValType;
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

    fn is_leaf(&self) -> bool {
        true
    }

    fn insert(&mut self, row: Self::Schema) -> bool {
        self.elements.insert(row)
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
        row: <Self::SuffixSchema as VariadicExt>::AsRefVar<'_>,
    ) -> Option<&'_ GhtLeaf<<Self as GeneralizedHashTrieNode>::Schema, Self::ValType>> {
        // TODO(mingwei): actually use the hash set as a hash set
        if self.elements.iter().any(|x| {
            let (_prefix, suffix) = Schema::split_by_suffix_ref(x.as_ref_var());
            <ValType as PartialEqVariadic>::eq_ref(row, suffix)
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

#[sealed]
/// iterators for HashTries based on a prefix search
pub trait HtPrefixIter<KeyPrefix> {
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
impl<'k, Head, Node, PrefixRest> HtPrefixIter<var_type!(&'k Head, ...PrefixRest)>
    for GhtInner<Head, Node>
where
    Head: Eq + Hash + Clone,
    Node: GeneralizedHashTrieNode + HtPrefixIter<PrefixRest>,
{
    type Item = <Node as HtPrefixIter<PrefixRest>>::Item;
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
impl<Head, Node> HtPrefixIter<var_type!()> for GhtInner<Head, Node>
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
impl<KeyPrefixRef, Schema, ValType> HtPrefixIter<KeyPrefixRef> for GhtLeaf<Schema, ValType>
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
    // Empty key, singleton val base case.
    (() => $z:ty => $( $schema:ty ),+ ) => (
        $crate::ght::GhtLeaf::<$( $schema ),*, $crate::variadics::var_type!( $z ) >
    );
    // Empty key, compound val base case.
    (() => $y:ty, $( $z:ty ),* => $( $schema:ty ),+ ) => (
        $crate::ght::GhtLeaf::<$( $schema ),*, ( $y, $crate::variadics::var_type!($( $z ),* )) >
    );
    // Singleton key, singleton val base case.
    ($a:ty => $z:ty  => $( $schema:ty ),+ ) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$( $schema ),*, $crate::variadics::var_type!( $z ) >>
    );
    // Singleton key, compound val base case.
    ($a:ty => $y:ty, $( $z:ty ),* => $( $schema:ty ),+ ) => (
            $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$( $schema ),*, $crate::variadics::var_type!($y, $( $z ),*) >>
    );
    // Recursive case.
    ($a:ty, $( $b:ty ),* => $( $z:ty ),* => $( $schema:ty ),+ ) => (
        $crate::ght::GhtInner::<$a, $crate::GhtNodeTypeWithSchema!($( $b ),* => $( $z ),* => $( $schema ),*)>
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
macro_rules! GhtNodeType {
    (() => $( $z:ty ),* ) => (
        $crate::GhtNodeTypeWithSchema!(() => $( $z ),* => $crate::variadics::var_type!($( $z ),* ))
    );
    ($a:ty => $( $z:ty ),*) => (
        $crate::GhtNodeTypeWithSchema!($a => $( $z ),* => $crate::variadics::var_type!($a, $( $z ),+ ))
    );
    ($a:ty, $( $b:ty ),* => $( $z:ty ),*) => (
        $crate::GhtNodeTypeWithSchema!($a, $( $b ),* => $( $z ),* => $crate::variadics::var_type!($a, $( $b ),*, $( $z ),*))
    );
}

/// Macro to create a GHT with the appropriate KeyType, ValType, Schema,
/// and a pointer to the GhtInner at the root of the Trie
#[macro_export]
macro_rules! GhtType {
    // ($a:ty => $( $z:ty ),* ) => (
    //     $crate::ght::GHT::<$crate::variadics::var_type!($a), $crate::variadics::var_type!($( $z ),*), $crate::GhtNodeType!($a => $( $z ),*)>
    // );
    // ($a:ty, $( $b:ty ),+  => $( $z:ty ),* ) => (
    //     $crate::ght::GHT::<$crate::variadics::var_type!( $a, $( $b ),+ ), $crate::variadics::var_type!($( $z ),*), $crate::GhtNodeType!($a, $( $b ),+ => $( $z ),*)>
    // );
    ($a:ty => $( $z:ty ),* ) => (
        $crate::ght::GHT::<$crate::variadics::var_type!($a), $crate::GhtNodeType!($a => $( $z ),*)>
    );
    ($a:ty, $( $b:ty ),+  => $( $z:ty ),* ) => (
        $crate::ght::GHT::<$crate::variadics::var_type!( $a, $( $b ),+ ), $crate::GhtNodeType!($a, $( $b ),+ => $( $z ),*)>
    );
}
