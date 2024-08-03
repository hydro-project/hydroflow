use std::cmp::Ordering::{self, *};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use sealed::sealed;
use variadics::{
    var_args, var_expr, var_type, AsRefVariadicPartialEq, Split, UnrefCloneVariadic, VariadicExt,
};

use crate::{IsBot, IsTop, LatticeBimorphism, LatticeOrd, Merge};

/// GeneralizedHashTrie wraps up a root GeneralizedHashTrieNode with metadata
/// for the key and value types associated with the full trie.
pub trait GeneralizedHashTrie {
    //+ for<'a> HtPrefixIter<var_type!(&'a Self::Head)> {
    /// Schema variadic: the type of rows we're storing
    type Schema: VariadicExt + AsRefVariadicPartialEq;

    /// the prefix of the Schema representing the Key type
    type KeyType: VariadicExt + AsRefVariadicPartialEq;
    /// the last column of the Schema, i.e. the Value type
    type ValType: VariadicExt + AsRefVariadicPartialEq;
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
    fn contains(&self, row: <Self::Schema as VariadicExt>::AsRefVar<'_>) -> bool;

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
}

/// GeneralizedHashTrie is a metadata node pointing to a root GeneralizedHashTrieNode.
#[derive(Debug, Clone)]
pub struct GHT<KeyType, ValType, Node>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Node: GeneralizedHashTrieNode,
{
    trie: Node,
    _key: std::marker::PhantomData<KeyType>,
    _val: std::marker::PhantomData<ValType>,
}

impl<K, V, Node> GHT<K, V, Node>
where
    K: VariadicExt + AsRefVariadicPartialEq,
    V: VariadicExt + AsRefVariadicPartialEq,
    Node: GeneralizedHashTrieNode,
{
    /// Just calls [`GeneralizedHashTrieNode::prefix_iter`].
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
    K: VariadicExt + AsRefVariadicPartialEq,
    V: VariadicExt + AsRefVariadicPartialEq,
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

    fn contains(&self, row: <Self::Schema as VariadicExt>::AsRefVar<'_>) -> bool {
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
}

impl<K, V, Node> Default for GHT<K, V, Node>
where
    K: VariadicExt + AsRefVariadicPartialEq,
    V: VariadicExt + AsRefVariadicPartialEq,
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

/// GeneralizedHashTrie node trait
#[sealed]
pub trait GeneralizedHashTrieNode: Default // + for<'a> HtPrefixIter<var_type!(&'a Self::Head)>
{
    /// Schema variadic: the type of rows we're storing
    type Schema: VariadicExt + AsRefVariadicPartialEq;
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
    fn contains(&self, row: <Self::Schema as VariadicExt>::AsRefVar<'_>) -> bool;

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
    children: HashMap<Head, Node>,
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

    fn contains(&self, row: <Self::Schema as VariadicExt>::AsRefVar<'_>) -> bool {
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
    T: Hash + Eq,
{
    elements: HashSet<T>,
}
impl<T> Default for GhtLeaf<T>
where
    T: Hash + Eq,
{
    fn default() -> Self {
        let elements = Default::default();
        Self { elements }
    }
}
#[sealed]
impl<T> GeneralizedHashTrieNode for GhtLeaf<T>
where
    T: 'static + Eq + VariadicExt + AsRefVariadicPartialEq + Hash,
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

    fn contains(&self, row: <Self::Schema as VariadicExt>::AsRefVar<'_>) -> bool {
        self.elements.iter().any(|r| r.as_ref_var_eq(row))
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
    T: Hash + Eq,
{
    fn from_iter<Iter: IntoIterator<Item = T>>(iter: Iter) -> Self {
        let elements = iter.into_iter().collect();
        Self { elements }
    }
}

impl<KeyType, ValType, Inner> Merge<GHT<KeyType, ValType, Inner>> for GHT<KeyType, ValType, Inner>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Inner: GeneralizedHashTrieNode + Merge<Inner>,
{
    fn merge(&mut self, other: GHT<KeyType, ValType, Inner>) -> bool {
        self.trie.merge(other.trie)
    }
}

impl<Head, Node> Merge<GhtInner<Head, Node>> for GhtInner<Head, Node>
where
    Node: GeneralizedHashTrieNode + Merge<Node>,
    Self: GeneralizedHashTrieNode,
    Head: Hash + Eq,
{
    fn merge(&mut self, other: GhtInner<Head, Node>) -> bool {
        let mut changed = false;

        for (k, v) in other.children.into_iter() {
            match self.children.entry(k) {
                std::collections::hash_map::Entry::Occupied(mut occupied) => {
                    changed |= occupied.get_mut().merge(v)
                }
                std::collections::hash_map::Entry::Vacant(vacant) => {
                    vacant.insert(v);
                    changed = true
                }
            }
        }
        changed
    }
}

impl<T> Merge<GhtLeaf<T>> for GhtLeaf<T>
where
    T: Hash + Eq,
{
    fn merge(&mut self, other: GhtLeaf<T>) -> bool {
        let old_len = self.elements.len();
        self.elements.extend(other.elements);
        self.elements.len() > old_len
    }
}

impl<KeyType, ValType, Inner> PartialEq<GHT<KeyType, ValType, Inner>>
    for GHT<KeyType, ValType, Inner>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Inner: GeneralizedHashTrieNode + Merge<Inner> + PartialEq,
{
    fn eq(&self, other: &GHT<KeyType, ValType, Inner>) -> bool {
        self.trie.eq(&other.trie)
    }
}

impl<Head, Node> PartialEq<GhtInner<Head, Node>> for GhtInner<Head, Node>
where
    Head: Hash + Eq + 'static,
    Node: GeneralizedHashTrieNode + 'static + PartialEq,
{
    fn eq(&self, other: &GhtInner<Head, Node>) -> bool {
        if self.children.len() != other.children.len() {
            return false;
        }

        for head in self.iter() {
            if other.get(head).is_none() {
                return false;
            }
            if !self.get(head).unwrap().eq(other.get(head).unwrap()) {
                return false;
            }
        }
        true
    }
}

impl<KeyType, ValType, Inner> PartialOrd<GHT<KeyType, ValType, Inner>>
    for GHT<KeyType, ValType, Inner>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Inner: GeneralizedHashTrieNode + Merge<Inner> + PartialOrd,
{
    fn partial_cmp(&self, other: &GHT<KeyType, ValType, Inner>) -> Option<Ordering> {
        self.trie.partial_cmp(&other.trie)
    }
}

impl<Head, Node> PartialOrd<GhtInner<Head, Node>> for GhtInner<Head, Node>
where
    Head: Hash + Eq + 'static,
    Node: 'static + GeneralizedHashTrieNode + PartialEq + PartialOrd,
{
    fn partial_cmp(&self, other: &GhtInner<Head, Node>) -> Option<Ordering> {
        if self.children.is_empty() && other.children.is_empty() {
            Some(Equal)
        } else {
            // across all keys, determine if we have stuff in self that's not in other
            // and vice versa
            let (selfonly, mut otheronly) =
                self.children
                    .iter()
                    .fold((false, false), |mut accum, (k, v)| {
                        if !other.children.contains_key(k) {
                            accum.0 = true; // selfonly is true
                            accum
                        } else {
                            match v.partial_cmp(other.children.get(k).unwrap()) {
                                Some(Greater) | None => {
                                    accum.0 = true; // selfonly is true
                                    accum
                                }
                                Some(Less) => {
                                    accum.1 = true; // otheronly is true
                                    accum
                                }
                                Some(Equal) => accum, // no changes
                            }
                        }
                    });
            // now check if other has keys that are missing in self
            otheronly |= !other.children.keys().all(|k| self.children.contains_key(k));

            if selfonly && otheronly {
                // unique stuff on both sides: order is incomparable
                None
            } else if selfonly && !otheronly {
                // unique stuff only in self
                Some(Greater)
            } else if !selfonly && otheronly {
                // unique stuff only in other
                Some(Less)
            } else {
                // nothing unique on either side
                Some(Equal)
            }
        }
    }
}

impl<T> PartialOrd<GhtLeaf<T>> for GhtLeaf<T>
where
    T: 'static + Hash + Eq,
{
    fn partial_cmp(&self, other: &GhtLeaf<T>) -> Option<Ordering> {
        match self.elements.len().cmp(&other.elements.len()) {
            Greater => {
                if other
                    .elements
                    .iter()
                    .all(|head| self.elements.contains(head))
                {
                    Some(Greater)
                } else {
                    None
                }
            }
            Equal => {
                if self
                    .elements
                    .iter()
                    .all(|head| other.elements.contains(head))
                {
                    Some(Equal)
                } else {
                    None
                }
            }
            Less => {
                if self
                    .elements
                    .iter()
                    .all(|head| other.elements.contains(head))
                {
                    Some(Less)
                } else {
                    None
                }
            }
        }
    }
}

impl<KeyType, ValType, Inner> LatticeOrd<GHT<KeyType, ValType, Inner>>
    for GHT<KeyType, ValType, Inner>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Inner: GeneralizedHashTrieNode,
    Self: PartialOrd<GHT<KeyType, ValType, Inner>>,
{
}

impl<Head, Node> LatticeOrd<GhtInner<Head, Node>> for GhtInner<Head, Node>
where
    Self: PartialOrd<GhtInner<Head, Node>>,
    Node: GeneralizedHashTrieNode,
{
}
impl<T> LatticeOrd<GhtLeaf<T>> for GhtLeaf<T>
where
    Self: PartialOrd<GhtLeaf<T>>,
    T: Hash + Eq,
{
}

impl<Head, Node> Eq for GhtInner<Head, Node>
where
    Node: GeneralizedHashTrieNode,
    Self: PartialEq,
{
}

impl<KeyType, ValType, Inner> IsBot for GHT<KeyType, ValType, Inner>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Inner: GeneralizedHashTrieNode + IsBot,
{
    fn is_bot(&self) -> bool {
        self.trie.is_bot()
    }
}

impl<Head, Node> IsBot for GhtInner<Head, Node>
where
    Node: GeneralizedHashTrieNode + IsBot,
{
    fn is_bot(&self) -> bool {
        self.children.iter().all(|(_, v)| v.is_bot())
    }
}

impl<T> IsBot for GhtLeaf<T>
where
    T: Hash + Eq,
{
    fn is_bot(&self) -> bool {
        self.elements.is_empty()
    }
}

impl<KeyType, ValType, Inner> IsTop for GHT<KeyType, ValType, Inner>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Inner: GeneralizedHashTrieNode + IsTop,
{
    fn is_top(&self) -> bool {
        self.trie.is_top()
    }
}

impl<Head, Node> IsTop for GhtInner<Head, Node>
where
    Node: GeneralizedHashTrieNode,
{
    fn is_top(&self) -> bool {
        false
    }
}

impl<T> IsTop for GhtLeaf<T>
where
    T: Hash + Eq,
{
    fn is_top(&self) -> bool {
        false
    }
}

#[sealed]
/// iterator for HashTries based on a prefix search
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
// #[sealed]
// impl<'k, KeyType, ValType, Head, Node, PrefixRest> HtPrefixIter<var_type!(&'k Head, ...PrefixRest)>
//     for GeneralizedHashTrie<KeyType, ValType, Inner>
// where
//     KeyType: VariadicExt + AsRefVariadicPartialEq,
//     ValType: VariadicExt + AsRefVariadicPartialEq,
//     Head: 'static + Hash + Eq,
//     Node: 'static + GeneralizedHashTrieNode + Default + HtPrefixIter<PrefixRest>,
//     <Node as HtPrefixIter<PrefixRest>>::Suffix: 'static, // meh
//     PrefixRest: 'static + Copy,                          // meh
// {
//     type Suffix = <Node as HtPrefixIter<PrefixRest>>::Suffix;
//     fn prefix_iter<'a>(
//         &'a self,
//         prefix: var_type!(&'k Head, ...PrefixRest),
//     ) -> impl Iterator<Item = <Self::Suffix as VariadicExt>::AsRefVar<'a>>
//     where
//         Self::Suffix: 'a,
//     {
//         self.trie.prefix_iter(prefix)
//     }
// }
#[sealed]
impl<'k, Head, Node, PrefixRest> HtPrefixIter<var_type!(&'k Head, ...PrefixRest)>
    for GhtInner<Head, Node>
where
    Head: Eq + Hash,
    Node: GeneralizedHashTrieNode + HtPrefixIter<PrefixRest>,
    <Node as HtPrefixIter<PrefixRest>>::Suffix: 'static, // meh
    PrefixRest: 'static + Copy,                          // meh
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
    Head: Eq + Hash,
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

/// Construct a Ght node type from the constituent key and
/// dependent column types. You pass it:
///    - a list of key column types and dependent column type separated by a fat arrow,
///         a la (K1, K2, K3 => T1, T2, T3)
/// This macro generates a hierarchy of GHT node types where each key column is associated with an GhtInner
/// of the associated column type, and the remaining dependent columns are associated with a variadic HTleaf
/// a la var_expr!(T1, T2, T3)
#[macro_export]
macro_rules! GhtNodeType {
    // Empty key base case.
    (() => $( $z:ty ),*) => (
        $crate::generalized_hash_trie::GhtLeaf::<$crate::variadics::var_type!($( $z ),* )>
    );
    // Singleton key base case.
    ($a:ty => $( $z:ty ),*) => (
        $crate::generalized_hash_trie::GhtInner::<$a, $crate::generalized_hash_trie::GhtLeaf::<$crate::variadics::var_type!($( $z ),*)>>
    );
    // Recursive case.
    ($a:ty, $( $b:ty ),* => $( $z:ty ),*) => (
        $crate::generalized_hash_trie::GhtInner::<$a, $crate::GhtNodeType!($( $b ),* => $( $z ),*)>
    );
}

/// Create a GHT with the appropriate KeyType, ValType, Schema, and a pointer to the GhtInner at the root of the Trie
#[macro_export]
macro_rules! GhtType {
    ($a:ty => $( $z:ty ),* ) => (
        $crate::generalized_hash_trie::GHT::<$crate::variadics::var_type!($a), $crate::variadics::var_type!($( $z ),*), $crate::GhtNodeType!($a => $( $z ),*)>
    );
    ($a:ty, $( $b:ty ),+  => $( $z:ty ),* ) => (
        $crate::generalized_hash_trie::GHT::<$crate::variadics::var_type!( $a, $( $b ),+ ), $crate::variadics::var_type!($( $z ),*), $crate::GhtNodeType!($a, $( $b ),+ => $( $z ),*)>
    );
}

//////////////////////////
/// BiMorphisms for GHT
//////////////////////////
/// Bimorphism for the cartesian product of two sets. Output is a set of all possible pairs of
/// items from the two input sets.
pub struct GhtCartesianProductBimorphism<GhtOut> {
    _phantom: std::marker::PhantomData<fn() -> GhtOut>,
}
impl<GhtOut> Default for GhtCartesianProductBimorphism<GhtOut> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}
impl<'a, 'b, GhtA, GhtB, GhtOut> LatticeBimorphism<&'a GhtA, &'b GhtB>
    for GhtCartesianProductBimorphism<GhtOut>
where
    GhtA: GeneralizedHashTrieNode,
    GhtB: GeneralizedHashTrieNode,
    GhtOut: FromIterator<var_type!(...GhtA::Schema, ...GhtB::Schema)>,
    <GhtA::Schema as VariadicExt>::AsRefVar<'a>: UnrefCloneVariadic,
    <GhtB::Schema as VariadicExt>::AsRefVar<'b>: UnrefCloneVariadic,
{
    type Output = GhtOut;

    fn call(&mut self, ght_a: &'a GhtA, ght_b: &'b GhtB) -> Self::Output {
        ght_a.recursive_iter().flat_map(|a| {
            ght_b
                .recursive_iter()
                .map(move |b| var_expr!(...UnrefCloneVariadic::clone_var(&a), ...UnrefCloneVariadic::clone_var(&b)))
        }).collect()
    }
}

/// Composable bimorphism, wraps an existing morphism by partitioning it per key.
///
/// For example, `GhtKeyedBimorphism<..., GhtCartesianProduct<...>>` is a join.
#[derive(Default)]
pub struct GhtBimorphism<Bimorphism> {
    bimorphism: Bimorphism,
    // _phantom: std::marker::PhantomData<fn() -> MapOut>,
}
impl<Bimorphism> GhtBimorphism<Bimorphism> {
    /// Create a `KeyedBimorphism` using `bimorphism` for handling values.
    pub fn new(bimorphism: Bimorphism) -> Self {
        Self {
            bimorphism,
            // _phantom: std::marker::PhantomData,
        }
    }
}

// impl<Head, GhtA, GhtB, ValFunc, Output> LatticeBimorphism<GhtA, GhtB>
//     for GhtKeyedBimorphism<ValFunc>
// where
//     Head: 'static + Clone + Hash + Eq,
//     GhtA: 'static + GeneralizedHashTrie<Head = Head>,
//     GhtB: 'static + GeneralizedHashTrie<Head = Head>,
//     GhtA::Node: 'static + Clone, // BAD: cloning subtrees
//     GhtB::Node: 'static + Clone, // BAD: cloning subtrees
//     // <GhtA::Node as GeneralizedHashTrieNode>::Get: Clone, // BAD: cloning subtrees
//     // <GhtB::Node as GeneralizedHashTrieNode>::Get: Clone, // BAD: cloning subtrees
//     for<'a, 'b> ValFunc: LatticeBimorphism<&'a GhtA::Node, &'b GhtB::Node, Output = Output>,
//     Output: GeneralizedHashTrieNode,
//     // for<'a> <GhtA::Schema as VariadicExt>::AsRefVar<'a>: UnrefCloneVariadic,
//     // for<'a> <GhtB::Schema as VariadicExt>::AsRefVar<'a>: UnrefCloneVariadic,
// {
//     type Output = GhtInner<Head, Output>; // HashMap<Head, ValFunc::Output>; // GhtOut;

//     fn call(&mut self, ght_a: GhtA, ght_b: GhtB) -> Self::Output {
//         let mut children = HashMap::<Head, ValFunc::Output>::new();
//         let trie_a = ght_a.get_trie();
//         let trie_b = ght_b.get_trie();
//         for head in trie_b.iter() {
//             if let Some(get_a) = trie_a.get(head) {
//                 let get_b = trie_b.get(head).unwrap();
//                 let val = self.bimorphism.call(get_a, get_b);
//                 children.insert(head.clone(), val);
//             }
//         }
//         GhtInner { children }
//         // GeneralizedHashTrie {
//         //     trie: GhtInner { children },
//         //     _key: std::marker::PhantomData,
//         //     _val: std::marker::PhantomData,
//         // }
//     }
// }

impl<GhtA, GhtB, ValFunc, Output> LatticeBimorphism<GhtA, GhtB> for GhtBimorphism<ValFunc>
where
    GhtA: GeneralizedHashTrie,
    GhtB: GeneralizedHashTrie,
    // // GhtA::Node: 'static + Clone, // BAD: cloning subtrees
    // // GhtB::Node: 'static + Clone, // BAD: cloning subtrees
    // // <GhtA::Node as GeneralizedHashTrieNode>::Get: Clone, // BAD: cloning subtrees
    // // <GhtB::Node as GeneralizedHashTrieNode>::Get: Clone, // BAD: cloning subtrees
    for<'a, 'b> ValFunc: LatticeBimorphism<&'a GhtA::Trie, &'b GhtB::Trie, Output = Output>,
    Output: GeneralizedHashTrieNode,
    // // for<'a> <GhtA::Schema as VariadicExt>::AsRefVar<'a>: UnrefCloneVariadic,
    // // for<'a> <GhtB::Schema as VariadicExt>::AsRefVar<'a>: UnrefCloneVariadic,
{
    type Output = GHT<(), (), Output>; // HashMap<Head, ValFunc::Output>; // GhtOut;

    fn call(&mut self, ght_a: GhtA, ght_b: GhtB) -> Self::Output {
        let node_bim = &mut self.bimorphism; // GhtNodeKeyedBimorphism::<ValFunc>::new(self.bimorphism);
        let trie = node_bim.call(ght_a.get_trie(), ght_b.get_trie());
        GHT {
            trie,
            _key: std::marker::PhantomData,
            _val: std::marker::PhantomData,
        }
    }
}

#[derive(Default)]
/// bimorphism trait for equijoining Ght Nodes
pub struct GhtNodeKeyedBimorphism<Bimorphism> {
    bimorphism: Bimorphism,
}
/// bimorphism implementation for equijoining Ght Nodes
impl<Bimorphism> GhtNodeKeyedBimorphism<Bimorphism> {
    /// initialize bimorphism
    pub fn new(bimorphism: Bimorphism) -> Self {
        Self { bimorphism }
    }
}
/// bimorphism implementation for equijoining Ght Nodes
impl<'a, 'b, Head, GhtA, GhtB, ValFunc> LatticeBimorphism<&'a GhtA, &'b GhtB>
    for GhtNodeKeyedBimorphism<ValFunc>
where
    Head: Clone + Hash + Eq,
    ValFunc: LatticeBimorphism<&'a GhtA::Get, &'b GhtB::Get>,
    ValFunc::Output: GeneralizedHashTrieNode,
    // MapA: MapIter + SimpleKeyedRef + SimpleCollectionRef,
    // MapB: for<'a> GetKeyValue<&'a MapA::Key, Key = MapA::Key> + SimpleCollectionRef,
    // MapA::Key: Clone + Eq,
    // MapA::Item: Clone,
    // MapB::Item: Clone,
    // MapOut: Default + MapInsert<MapA::Key> + Collection<Item = ValFunc::Output>,
    GhtA: GeneralizedHashTrieNode<Head = Head>,
    GhtB: GeneralizedHashTrieNode<Head = Head>,
    // GhtOut: FromIterator<(Head, ValFunc::Output)>,
    <GhtA::Schema as VariadicExt>::AsRefVar<'a>: UnrefCloneVariadic,
    <GhtB::Schema as VariadicExt>::AsRefVar<'b>: UnrefCloneVariadic,
{
    type Output = GhtInner<Head, ValFunc::Output>; // HashMap<Head, ValFunc::Output>; // GhtOut;

    fn call(&mut self, ght_a: &'a GhtA, ght_b: &'b GhtB) -> Self::Output {
        let mut children = HashMap::<Head, ValFunc::Output>::new();
        for head in ght_b.iter() {
            if let Some(get_a) = ght_a.get(head) {
                let get_b = ght_b.get(head).unwrap();
                let val = self.bimorphism.call(get_a, get_b);
                children.insert(head.clone(), val);
            }
        }
        GhtInner { children }
    }
}

/// bimorphism trait for equijoin on full tuple (keys in all GhtInner nodes)
pub trait DeepJoinLatticeBimorphism {
    /// bimorphism type for equijoin on full tuple (keys in all GhtInner nodes)
    type DeepJoinLatticeBimorphism;
}
/// bimorphism implementation for equijoin on full tuple (keys in all GhtInner nodes)
impl<Head, NodeA, NodeB> DeepJoinLatticeBimorphism
    for (GhtInner<Head, NodeA>, GhtInner<Head, NodeB>)
where
    Head: 'static + Hash + Eq + Clone,
    NodeA: 'static + GeneralizedHashTrieNode,
    NodeB: 'static + GeneralizedHashTrieNode,
    (NodeA, NodeB): DeepJoinLatticeBimorphism,
{
    type DeepJoinLatticeBimorphism = GhtNodeKeyedBimorphism<
        <(NodeA, NodeB) as DeepJoinLatticeBimorphism>::DeepJoinLatticeBimorphism,
    >;
}
impl<A, B> DeepJoinLatticeBimorphism for (GhtLeaf<A>, GhtLeaf<B>)
where
    A: 'static + VariadicExt + Eq + Hash + AsRefVariadicPartialEq,
    B: 'static + VariadicExt + Eq + Hash + AsRefVariadicPartialEq,
    for<'x> A::AsRefVar<'x>: UnrefCloneVariadic,
    for<'x> B::AsRefVar<'x>: UnrefCloneVariadic,
    var_type!(...A, ...B): Eq + Hash,
{
    type DeepJoinLatticeBimorphism = GhtCartesianProductBimorphism<GhtLeaf<var_type!(...A, ...B)>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NaiveLatticeOrd;

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

        let inner = ht_root.trie.get(&42).unwrap();
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
        let inner_key = ht_root.trie.iter().next().unwrap();
        let inner = ht_root.trie.get(inner_key).unwrap();
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

        for row in htrie.prefix_iter(var_expr!(&42)) {
            println!("42: {:?}", row);
        }

        for row in htrie.prefix_iter(var_expr!(&42, &315)) {
            println!("42,315: {:?}", row);
        }

        assert!(htrie.prefix_iter(var_expr!(&42, &315)).any(|_| true));
        // // Too long:
        // for row in htrie.prefix_iter(var_expr!(&42, &315, &43770)) {
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
        let ght_out = bim.call(&ght_a.trie, &ght_b.trie);
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
        trie1.trie.insert(var_expr!(1, 2, 3));
        let mut trie2 = MyRoot::default();
        trie2.trie.insert(var_expr!(1, 2, 4));

        let tup1 = trie1.trie.recursive_iter_keys().next().unwrap();
        let (k1, _v1) = MyRoot::split_key_val(tup1);
        let tup2 = trie2.trie.recursive_iter_keys().next().unwrap();
        let (k2, _v2) = MyRoot::split_key_val(tup2);
        assert!(tup1 != tup2);
        assert_eq!(k1, k2);
    }
}
