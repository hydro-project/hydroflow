use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use sealed::sealed;
use variadics::{var_args, var_expr, var_type, Split, VariadicExt};

use crate::ght_lattice::DeepJoinLatticeBimorphism;

/// GeneralizedHashTrie wraps up a root GeneralizedHashTrieNode with metadata
/// for the key and value types associated with the full trie.
pub trait GeneralizedHashTrie {
    //+ for<'a> HtPrefixIter<var_type!(&'a Self::Head)> {
    /// Schema variadic: the type of rows we're storing
    type Schema: VariadicExt;

    /// the prefix of the Schema representing the Key type
    type KeyType: VariadicExt;
    /// the last column of the Schema, i.e. the Value type
    type ValType: VariadicExt;
    // /// The type of the first column in the Schema
    // type Head: Eq + Hash;
    // /// The type of the Node in the root
    // type Node: GeneralizedHashTrieNode;
    /// The underlying root Trie Node
    type Trie: GeneralizedHashTrieNode;

    /// Create a new Ght from the iterator.
    fn new_from(input: impl IntoIterator<Item = Self::Schema>) -> Self;

    /// Report the height of the tree if its not empty. This is the length of a root to leaf path -1.
    /// E.g. if we have GhtInner<GhtInner<GhtLeaf...>> the height is 2
    fn height(&self) -> Option<usize>;

    /// Inserts items into the hash trie.
    fn insert(&mut self, row: Self::Schema) -> bool;

    /// Returns `true` if the (entire) row is found in the trie, `false` otherwise.
    fn contains<'a>(&'a self, row: <Self::Schema as VariadicExt>::AsRefVar<'a>) -> bool;

    /// Iterate through the (entire) rows stored in this HashTrie.
    /// Returns Variadics, not tuples.
    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>>;

    /// Iterate through the (entire) rows stored in this HashTrie, but with the leaf
    /// values stubbed out ((), ()).
    fn recursive_iter_keys(
        &self,
    ) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>>;

    // /// Extract Key and Value from a by-value tuple
    // fn split_key_val_no_refs(tup: Self::Schema) -> (Self::KeyType, Self::ValType)
    // where
    //     Self::Schema: Split<Self::KeyType, Suffix = Self::ValType>,
    // {
    //     tup.split()
    // }

    /// Extract Key and Value from a returned tuple
    fn split_key_val<'a>(
        tup: <Self::Schema as VariadicExt>::AsRefVar<'a>,
    ) -> (
        <Self::KeyType as VariadicExt>::AsRefVar<'a>,
        <Self::ValType as VariadicExt>::AsRefVar<'a>,
    )
    where
        <Self::Schema as VariadicExt>::AsRefVar<'a>: Split<
            <Self::KeyType as VariadicExt>::AsRefVar<'a>,
            Suffix = <Self::ValType as VariadicExt>::AsRefVar<'a>,
        >,
    {
        tup.split()
    }

    /// get a ref to the underlying root note of the trie
    fn get_trie(&self) -> &Self::Trie;

    /// get a mutable ref to the underlying root note of the trie
    fn get_mut_trie(&mut self) -> &mut Self::Trie;
}

/// GeneralizedHashTrie is a metadata node pointing to a root GeneralizedHashTrieNode.
#[derive(Debug, Clone)]
pub struct GHT<KeyType, ValType, Node>
where
    KeyType: VariadicExt, // + AsRefVariadicPartialEq
    ValType: VariadicExt, // + AsRefVariadicPartialEq
    Node: GeneralizedHashTrieNode,
{
    pub(crate) trie: Node,
    pub(crate) _key: std::marker::PhantomData<KeyType>,
    pub(crate) _val: std::marker::PhantomData<ValType>,
}

impl<K, V, Node> GHT<K, V, Node>
where
    K: VariadicExt, // + AsRefVariadicPartialEq
    V: VariadicExt, // + AsRefVariadicPartialEq
    Node: GeneralizedHashTrieNode,
{
    /// Just calls `prefix_iter` on the underlying trie.
    pub fn prefix_iter<'a, Prefix>(
        &'a self,
        prefix: Prefix,
    ) -> impl Iterator<Item = <Node::Suffix as VariadicExt>::AsRefVar<'a>>
    where
        Node: HtPrefixIter<Prefix>,
        Prefix: 'a,
    {
        self.trie.prefix_iter(prefix)
    }
}

impl<K, V, Node> GeneralizedHashTrie for GHT<K, V, Node>
where
    K: VariadicExt, // + AsRefVariadicPartialEq
    V: VariadicExt, // + AsRefVariadicPartialEq
    Node: GeneralizedHashTrieNode,
{
    type KeyType = K;
    type ValType = V;
    type Schema = Node::Schema;
    type Trie = Node;

    fn new_from(input: impl IntoIterator<Item = Self::Schema>) -> Self {
        let trie = GeneralizedHashTrieNode::new_from(input);
        GHT {
            trie,
            _key: Default::default(),
            _val: Default::default(),
        }
    }

    fn height(&self) -> Option<usize> {
        self.trie.height()
    }

    fn insert(&mut self, row: Self::Schema) -> bool {
        self.trie.insert(row)
    }

    fn contains<'a>(&'a self, row: <Self::Schema as VariadicExt>::AsRefVar<'a>) -> bool {
        self.trie.contains(row)
    }

    /// Iterate through the (entire) rows stored in this HashTrie.
    /// Returns Variadics, not tuples.
    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
        self.trie.recursive_iter()
    }

    /// Iterate through the (entire) rows stored in this HashTrie, but with the leaf
    /// values stubbed out ((), ()).
    fn recursive_iter_keys(
        &self,
    ) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
        self.trie.recursive_iter_keys()
    }

    fn get_trie(&self) -> &Node {
        &self.trie
    }

    /// get a mutable ref to the underlying root note of the trie
    fn get_mut_trie(&mut self) -> &mut Self::Trie {
        &mut self.trie
    }
}

impl<K, V, Node> Default for GHT<K, V, Node>
where
    K: VariadicExt, // + AsRefVariadicPartialEq
    V: VariadicExt, // + AsRefVariadicPartialEq
    Node: GeneralizedHashTrieNode,
{
    fn default() -> Self {
        let tree = Node::default();
        let _key: std::marker::PhantomData<K> = Default::default();
        let _val: std::marker::PhantomData<V> = Default::default();
        Self {
            trie: tree,
            _key,
            _val,
        }
    }
}

/// GeneralizedHashTrieNode trait
#[sealed]
pub trait GeneralizedHashTrieNode: Default // + for<'a> HtPrefixIter<var_type!(&'a Self::Head)>
{
    /// Schema variadic: the type of rows we're storing in this subtrie
    type Schema: VariadicExt;
    /// The type of the first column in the Schema
    type Head: Eq + Hash;

    /// Create a new Ght from the iterator.
    fn new_from(input: impl IntoIterator<Item = Self::Schema>) -> Self;

    /// Report the height of the tree if its not empty. This is the length of a root to leaf path -1.
    /// E.g. if we have GhtInner<GhtInner<GhtLeaf...>> the height is 2
    fn height(&self) -> Option<usize>;

    /// Inserts items into the hash trie.
    fn insert(&mut self, row: Self::Schema) -> bool;

    /// Returns `true` if the (entire) row is found in the trie, `false` otherwise.
    /// See `get()` below to look just for "head" keys in this node
    fn contains<'a>(&'a self, row: <Self::Schema as VariadicExt>::AsRefVar<'a>) -> bool;

    /// Iterator for the "head" keys (from inner nodes) or elements (from leaf nodes).
    fn iter(&self) -> impl Iterator<Item = &'_ Self::Head>;

    /// Type returned by [`Self::get`].
    type Get: GeneralizedHashTrieNode;

    /// On an Inner node, retrieves the value (child) associated with the given "head" key.
    /// returns an `Option` containing a reference to the value if found, or `None` if not found.
    /// On a Leaf node, returns None.
    fn get(&self, head: &Self::Head) -> Option<&'_ Self::Get>;

    /// Iterate through the (entire) rows stored in this HashTrie.
    /// Returns Variadics, not tuples.
    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>>;

    /// Iterate through the (entire) rows stored in this HashTrie, but with the leaf
    /// values stubbed out ((), ()).
    fn recursive_iter_keys(
        &self,
    ) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>>;

    /// Bimorphism for joining on full tuple keys (all GhtInner keys) in the trie
    type DeepJoin<Other>
    where
        Other: GeneralizedHashTrieNode,
        (Self, Other): DeepJoinLatticeBimorphism;
}

/// internal node of a HashTrie
#[derive(Debug, Clone)]
pub struct GhtInner<Head, Node>
where
    Node: GeneralizedHashTrieNode,
{
    pub(crate) children: HashMap<Head, Node>,
}
impl<Head, Node: GeneralizedHashTrieNode> Default for GhtInner<Head, Node>
where
    Node: GeneralizedHashTrieNode,
{
    fn default() -> Self {
        let children = Default::default();
        Self { children }
    }
}
#[sealed]
impl<Head, Node> GeneralizedHashTrieNode for GhtInner<Head, Node>
where
    Head: 'static + Hash + Eq,
    Node: 'static + GeneralizedHashTrieNode,
{
    type Schema = var_type!(Head, ...Node::Schema);
    type Head = Head;

    fn new_from(input: impl IntoIterator<Item = Self::Schema>) -> Self {
        let mut retval: Self = Default::default();
        for i in input {
            retval.insert(i);
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

    fn insert(&mut self, row: Self::Schema) -> bool {
        let var_args!(head, ...rest) = row;
        self.children.entry(head).or_default().insert(rest)
    }

    fn contains<'a>(&'a self, row: <Self::Schema as VariadicExt>::AsRefVar<'a>) -> bool {
        let var_args!(head, ...rest) = row;
        if let Some(node) = self.children.get(head) {
            node.contains(rest)
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
            .flat_map(|(k, vs)| vs.recursive_iter().map(move |v| var_expr!(k, ...v)))
    }

    fn recursive_iter_keys(
        &self,
    ) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
        self.children
            .iter()
            .flat_map(|(k, vs)| vs.recursive_iter_keys().map(move |v| var_expr!(k, ...v)))
    }

    type DeepJoin<Other> = <(Self, Other) as DeepJoinLatticeBimorphism>::DeepJoinLatticeBimorphism
    where
        Other: GeneralizedHashTrieNode,
        (Self, Other): DeepJoinLatticeBimorphism;
}
impl<Head, Node> FromIterator<var_type!(Head, ...Node::Schema)> for GhtInner<Head, Node>
where
    Head: 'static + Hash + Eq,
    Node: 'static + GeneralizedHashTrieNode,
{
    fn from_iter<Iter: IntoIterator<Item = var_type!(Head, ...Node::Schema)>>(iter: Iter) -> Self {
        let mut out = Self::default();
        for row in iter {
            out.insert(row);
        }
        out
    }
}

/// leaf node of a HashTrie
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GhtLeaf<T>
where
    T: Eq + Hash,
{
    pub(crate) elements: HashSet<T>,
}
impl<T> Default for GhtLeaf<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        let elements = Default::default();
        Self { elements }
    }
}
#[sealed]
impl<T> GeneralizedHashTrieNode for GhtLeaf<T>
where
    T: 'static + Eq + VariadicExt + Hash,
    for<'a> T::AsRefVar<'a>: PartialEq,
{
    type Schema = T;
    type Head = T;

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

    fn insert(&mut self, row: Self::Schema) -> bool {
        self.elements.insert(row)
    }

    fn contains<'a>(&'a self, row: <Self::Schema as VariadicExt>::AsRefVar<'a>) -> bool {
        self.elements.iter().any(|r| r.as_ref_var() == row)
    }

    fn iter(&self) -> impl Iterator<Item = &'_ Self::Head> {
        self.elements.iter()
    }

    type Get = Self;
    fn get(&self, _head: &Self::Head) -> Option<&'_ Self::Get> {
        Option::<&Self>::None
    }

    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
        self.elements.iter().map(T::as_ref_var)
    }

    fn recursive_iter_keys(
        &self,
    ) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
        let out = self.elements.iter().map(T::as_ref_var).next().unwrap();
        std::iter::once(out)
    }

    type DeepJoin<Other> = <(Self, Other) as DeepJoinLatticeBimorphism>::DeepJoinLatticeBimorphism
    where
        Other: GeneralizedHashTrieNode,
        (Self, Other): DeepJoinLatticeBimorphism;
}

impl<T> FromIterator<T> for GhtLeaf<T>
where
    T: Eq + Hash,
{
    fn from_iter<Iter: IntoIterator<Item = T>>(iter: Iter) -> Self {
        let elements = iter.into_iter().collect();
        Self { elements }
    }
}

#[sealed]
/// iterators for HashTries based on a prefix search
pub trait HtPrefixIter<Prefix> {
    /// type of the suffix of this prefix
    type Suffix: VariadicExt;
    /// given a prefix, return an iterator through the items below
    fn prefix_iter<'a>(
        &'a self,
        prefix: Prefix,
    ) -> impl Iterator<Item = <Self::Suffix as VariadicExt>::AsRefVar<'a>>
    where
        Self::Suffix: 'a;
}

#[sealed]
impl<'k, Head, Node, PrefixRest> HtPrefixIter<var_type!(&'k Head, ...PrefixRest)>
    for GhtInner<Head, Node>
where
    Head: Eq + Hash,
    Node: GeneralizedHashTrieNode + HtPrefixIter<PrefixRest>,
    PrefixRest: Copy,
{
    type Suffix = <Node as HtPrefixIter<PrefixRest>>::Suffix;
    fn prefix_iter<'a>(
        &'a self,
        prefix: var_type!(&'k Head, ...PrefixRest),
    ) -> impl Iterator<Item = <Self::Suffix as VariadicExt>::AsRefVar<'a>>
    where
        Self::Suffix: 'a,
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
impl<'k, Head> HtPrefixIter<var_type!(&'k Head)> for GhtLeaf<Head>
where
    Head: 'static + Eq + VariadicExt + Hash,
    for<'a, 'b> Head::AsRefVar<'a>: PartialEq<Head::AsRefVar<'b>>,
{
    type Suffix = var_expr!();
    fn prefix_iter<'a>(
        &'a self,
        prefix: var_type!(&'k Head),
    ) -> impl Iterator<Item = <Self::Suffix as VariadicExt>::AsRefVar<'a>>
    where
        Self::Suffix: 'a,
    {
        let var_args!(head) = prefix;
        self.elements.contains(head).then_some(()).into_iter()
    }
}
#[sealed]
impl<This> HtPrefixIter<var_type!()> for This
where
    This: 'static + GeneralizedHashTrieNode,
{
    type Suffix = <Self as GeneralizedHashTrieNode>::Schema;
    fn prefix_iter<'a>(
        &'a self,
        _prefix: var_type!(),
    ) -> impl Iterator<Item = <Self::Suffix as VariadicExt>::AsRefVar<'a>>
    where
        Self::Suffix: 'a,
    {
        self.recursive_iter()
    }
}

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
    // Empty key base case.
    (() => $( $z:ty ),*) => (
        $crate::ght::GhtLeaf::<$crate::variadics::var_type!($( $z ),* )>
    );
    // Singleton key base case.
    ($a:ty => $( $z:ty ),*) => (
        $crate::ght::GhtInner::<$a, $crate::ght::GhtLeaf::<$crate::variadics::var_type!($( $z ),*)>>
    );
    // Recursive case.
    ($a:ty, $( $b:ty ),* => $( $z:ty ),*) => (
        $crate::ght::GhtInner::<$a, $crate::GhtNodeType!($( $b ),* => $( $z ),*)>
    );
}

/// Macro to create a GHT with the appropriate KeyType, ValType, Schema,
/// and a pointer to the GhtInner at the root of the Trie
#[macro_export]
macro_rules! GhtType {
    ($a:ty => $( $z:ty ),* ) => (
        $crate::ght::GHT::<$crate::variadics::var_type!($a), $crate::variadics::var_type!($( $z ),*), $crate::GhtNodeType!($a => $( $z ),*)>
    );
    ($a:ty, $( $b:ty ),+  => $( $z:ty ),* ) => (
        $crate::ght::GHT::<$crate::variadics::var_type!( $a, $( $b ),+ ), $crate::variadics::var_type!($( $z ),*), $crate::GhtNodeType!($a, $( $b ),+ => $( $z ),*)>
    );
}
