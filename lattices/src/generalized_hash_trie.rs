use std::cmp::Ordering::{self, *};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use sealed::sealed;
use variadics::{
    var_args, var_expr, var_type, AsRefVariadicPartialEq, UnrefCloneVariadic, VariadicExt,
};

use crate::{IsBot, IsTop, LatticeBimorphism, LatticeOrd, Merge};

/// GeneralizedHashTrieTrait wraps up a root GeneralizedHashTrieNode with metadata
/// for the key and value types associated with the full trie.
trait GeneralizedHashTrieTrait: Default + for<'a> HtPrefixIter<var_type!(&'a Self::Head)> {
    /// Schema variadic: the type of rows we're storing
    type Schema: VariadicExt + AsRefVariadicPartialEq;
    /// the prefix of the Schema representing the Key type
    type KeyType: VariadicExt + AsRefVariadicPartialEq;
    /// the last column of the Schema, i.e. the Value type
    type ValType: VariadicExt + AsRefVariadicPartialEq;
    /// The type of the first column in the Schema
    type Head: Eq + Hash;

    /// Create a new, empty Ght
    fn new_from(input: Vec<Self::Schema>) -> Self;

    /// Report the height of the tree if its not empty. This is the length of a root to leaf path -1.
    /// E.g. if we have HtInner<HtInner<HtLeaf...>> the height is 2
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
}
/// GeneralizedHashTrie is a metadata node pointing to a root GeneralizedHashTrieNode.
#[derive(Debug, Clone)]
pub struct GeneralizedHashTrie<KeyType, ValType, Head, Node>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Head: 'static + Hash + Eq,
    Node: 'static + GeneralizedHashTrieNode + Default + Merge<Node>,
{
    trie: HtInner<Head, Node>,
    _key: std::marker::PhantomData<KeyType>,
    _val: std::marker::PhantomData<ValType>,
}

impl<K, V, Head, Node> GeneralizedHashTrieTrait for GeneralizedHashTrie<K, V, Head, Node>
where
    K: VariadicExt + AsRefVariadicPartialEq,
    V: VariadicExt + AsRefVariadicPartialEq,
    Head: 'static + Hash + Eq,
    Node: 'static + GeneralizedHashTrieNode + Default + Merge<Node>,
{
    type KeyType = K;
    type ValType = V;
    type Schema = var_type!(Head, ...Node::Schema);
    type Head = Head;

    fn new_from(input: Vec<Self::Schema>) -> Self {
        let trie = GeneralizedHashTrieNode::new_from(input);
        GeneralizedHashTrie {
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
}

impl<K, V, Head, Node> Default for GeneralizedHashTrie<K, V, Head, Node>
where
    K: VariadicExt + AsRefVariadicPartialEq,
    V: VariadicExt + AsRefVariadicPartialEq,
    Head: 'static + Hash + Eq,
    Node: 'static + GeneralizedHashTrieNode + Default + Merge<Node>,
{
    fn default() -> Self {
        let tree = HtInner::<Head, Node>::default();
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
pub trait GeneralizedHashTrieNode:
    Default + for<'a> HtPrefixIter<var_type!(&'a Self::Head)>
{
    /// Schema variadic: the type of rows we're storing
    type Schema: VariadicExt + AsRefVariadicPartialEq;
    /// The type of the first column in the Schema
    type Head: Eq + Hash;

    /// Create a new, empty Ght
    fn new_from(input: Vec<Self::Schema>) -> Self;

    /// Report the height of the tree if its not empty. This is the length of a root to leaf path -1.
    /// E.g. if we have HtInner<HtInner<HtLeaf...>> the height is 2
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
}

/// internal node of a HashTrie
#[derive(Debug, Clone)]
pub struct HtInner<Head, Node>
where
    Node: GeneralizedHashTrieNode,
{
    children: HashMap<Head, Node>,
}
impl<Head, Node: GeneralizedHashTrieNode> Default for HtInner<Head, Node>
where
    Node: GeneralizedHashTrieNode,
{
    fn default() -> Self {
        let children = Default::default();
        Self { children }
    }
}
#[sealed]
impl<Head, Node> GeneralizedHashTrieNode for HtInner<Head, Node>
where
    Head: 'static + Hash + Eq,
    Node: 'static + GeneralizedHashTrieNode + Merge<Node>,
{
    type Schema = var_type!(Head, ...Node::Schema);
    type Head = Head;

    fn new_from(input: Vec<Self::Schema>) -> Self {
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
}
impl<Head, Node> FromIterator<var_type!(Head, ...Node::Schema)> for HtInner<Head, Node>
where
    Head: 'static + Hash + Eq,
    Node: 'static + GeneralizedHashTrieNode + Merge<Node>,
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
pub struct HtLeaf<T>
where
    T: Hash + Eq,
{
    elements: HashSet<T>,
}
impl<T> Default for HtLeaf<T>
where
    T: Hash + Eq,
{
    fn default() -> Self {
        let elements = Default::default();
        Self { elements }
    }
}
#[sealed]
impl<T> GeneralizedHashTrieNode for HtLeaf<T>
where
    T: 'static + Eq + VariadicExt + AsRefVariadicPartialEq + Hash,
{
    type Schema = T;
    type Head = T;

    fn new_from(input: Vec<Self::Schema>) -> Self {
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
}

impl<T> FromIterator<T> for HtLeaf<T>
where
    T: Hash + Eq,
{
    fn from_iter<Iter: IntoIterator<Item = T>>(iter: Iter) -> Self {
        let elements = iter.into_iter().collect();
        Self { elements }
    }
}

impl<KeyType, ValType, Head, Node> Merge<GeneralizedHashTrie<KeyType, ValType, Head, Node>>
    for GeneralizedHashTrie<KeyType, ValType, Head, Node>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Node: GeneralizedHashTrieNode + Merge<Node>,
    Head: Hash + Eq,
{
    fn merge(&mut self, other: GeneralizedHashTrie<KeyType, ValType, Head, Node>) -> bool {
        self.trie.merge(other.trie)
    }
}

impl<Head, Node> Merge<HtInner<Head, Node>> for HtInner<Head, Node>
where
    Node: GeneralizedHashTrieNode + Merge<Node>,
    Self: GeneralizedHashTrieNode,
    Head: Hash + Eq,
{
    fn merge(&mut self, other: HtInner<Head, Node>) -> bool {
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

impl<T> Merge<HtLeaf<T>> for HtLeaf<T>
where
    T: Hash + Eq,
{
    fn merge(&mut self, other: HtLeaf<T>) -> bool {
        let old_len = self.elements.len();
        self.elements.extend(other.elements);
        self.elements.len() > old_len
    }
}

impl<KeyType, ValType, Head, Node> PartialEq<GeneralizedHashTrie<KeyType, ValType, Head, Node>>
    for GeneralizedHashTrie<KeyType, ValType, Head, Node>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Node: GeneralizedHashTrieNode + Merge<Node> + PartialEq,
    Self: GeneralizedHashTrieNode,
    Head: Hash + Eq,
{
    fn eq(&self, other: &GeneralizedHashTrie<KeyType, ValType, Head, Node>) -> bool {
        self.trie.eq(&other.trie)
    }
}

impl<Head, Node> PartialEq<HtInner<Head, Node>> for HtInner<Head, Node>
where
    Head: Hash + Eq + 'static,
    Node: GeneralizedHashTrieNode + 'static + PartialEq + Merge<Node>,
{
    fn eq(&self, other: &HtInner<Head, Node>) -> bool {
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

impl<KeyType, ValType, Head, Node> PartialOrd<GeneralizedHashTrie<KeyType, ValType, Head, Node>>
    for GeneralizedHashTrie<KeyType, ValType, Head, Node>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Node: GeneralizedHashTrieNode + Merge<Node> + PartialEq + PartialOrd,
    Self: GeneralizedHashTrieNode,
    Head: Hash + Eq,
{
    fn partial_cmp(
        &self,
        other: &GeneralizedHashTrie<KeyType, ValType, Head, Node>,
    ) -> Option<Ordering> {
        self.trie.partial_cmp(&other.trie)
    }
}

impl<Head, Node> PartialOrd<HtInner<Head, Node>> for HtInner<Head, Node>
where
    Head: Hash + Eq + 'static,
    Node: GeneralizedHashTrieNode + PartialEq + Merge<Node> + 'static + PartialOrd,
{
    fn partial_cmp(&self, other: &HtInner<Head, Node>) -> Option<Ordering> {
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

impl<T> PartialOrd<HtLeaf<T>> for HtLeaf<T>
where
    T: Hash + Eq + 'static,
{
    fn partial_cmp(&self, other: &HtLeaf<T>) -> Option<Ordering> {
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

impl<KeyType, ValType, Head, Node> LatticeOrd<GeneralizedHashTrie<KeyType, ValType, Head, Node>>
    for GeneralizedHashTrie<KeyType, ValType, Head, Node>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Head: Hash + Eq,
    Node: GeneralizedHashTrieNode + Merge<Node>,
    Self: PartialOrd<GeneralizedHashTrie<KeyType, ValType, Head, Node>>,
{
}

impl<Head, Node> LatticeOrd<HtInner<Head, Node>> for HtInner<Head, Node>
where
    Self: PartialOrd<HtInner<Head, Node>>,
    Node: GeneralizedHashTrieNode,
{
}
impl<T> LatticeOrd<HtLeaf<T>> for HtLeaf<T>
where
    Self: PartialOrd<HtLeaf<T>>,
    T: Hash + Eq,
{
}

impl<Head, Node> Eq for HtInner<Head, Node>
where
    Node: GeneralizedHashTrieNode,
    Self: PartialEq,
{
}

impl<KeyType, ValType, Head, Node> IsBot for GeneralizedHashTrie<KeyType, ValType, Head, Node>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Head: Hash + Eq,
    Node: GeneralizedHashTrieNode + Merge<Node> + IsBot,
{
    fn is_bot(&self) -> bool {
        self.trie.is_bot()
    }
}

impl<Head, Node> IsBot for HtInner<Head, Node>
where
    Node: GeneralizedHashTrieNode + IsBot,
{
    fn is_bot(&self) -> bool {
        self.children.iter().all(|(_, v)| v.is_bot())
    }
}

impl<T> IsBot for HtLeaf<T>
where
    T: Hash + Eq,
{
    fn is_bot(&self) -> bool {
        self.elements.is_empty()
    }
}

impl<KeyType, ValType, Head, Node> IsTop for GeneralizedHashTrie<KeyType, ValType, Head, Node>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Head: Hash + Eq,
    Node: GeneralizedHashTrieNode + Merge<Node> + IsTop,
{
    fn is_top(&self) -> bool {
        self.trie.is_top()
    }
}

impl<Head, Node> IsTop for HtInner<Head, Node>
where
    Node: GeneralizedHashTrieNode,
{
    fn is_top(&self) -> bool {
        false
    }
}

impl<T> IsTop for HtLeaf<T>
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
#[sealed]
impl<'k, KeyType, ValType, Head, Node, PrefixRest> HtPrefixIter<var_type!(&'k Head, ...PrefixRest)>
    for GeneralizedHashTrie<KeyType, ValType, Head, Node>
where
    KeyType: VariadicExt + AsRefVariadicPartialEq,
    ValType: VariadicExt + AsRefVariadicPartialEq,
    Head: 'static + Hash + Eq,
    Node: 'static + GeneralizedHashTrieNode + Default + Merge<Node> + HtPrefixIter<PrefixRest>,
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
        self.trie.prefix_iter(prefix)
    }
}
#[sealed]
impl<'k, Head, Node, PrefixRest> HtPrefixIter<var_type!(&'k Head, ...PrefixRest)>
    for HtInner<Head, Node>
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
impl<'k, Head> HtPrefixIter<var_type!(&'k Head)> for HtLeaf<Head>
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
/// dependent column types. You pass it either:
///    - a flat tuple of types, a la (T, U, V)
///    - or a list of key column types and dependent column type separated by a fat arrow,
///         a la (K1, K2, K3 => T1, T2, T3)
/// This macro generates a ght where each key column is associated with a level of HtInner
/// nodes, and the remaining dependent columns are associated with a variadic HTleaf level
/// a la var_expr!(T1, T2, T3)
#[macro_export]
macro_rules! GhtNodeType {
    // Flat base case.
    ($a:ty) => {
        HtLeaf::<var_type!($a)>
    };
    // Flat recursive case.
    ($a:ty, $( $b:ty ),* ) => (
        HtInner::<$a, GhtNodeType!($( $b ),*)>
    );

    // Arrow base case.
    (() => $( $z:ty ),*) => (
        HtLeaf::<var_type!($( $z ),* )>
    );
    // Arrow base case.
    ($a:ty => $( $z:ty ),*) => (
        HtInner::<$a, HtLeaf::<var_type!($( $z ),*)>>
    );
    // Arrow recursive case.
    ($a:ty, $( $b:ty ),* => $( $z:ty ),*) => (
        HtInner::<$a, GhtNodeType!($( $b ),* => $( $z ),*)>
    );
}

#[macro_export]
/// Create the type of a GHT
macro_rules! Ght {
    ($a:ty => $( $z:ty ),* ) => (
        GeneralizedHashTrie::<var_type!($a), var_type!($( $z ),*), $a, GhtNodeType!(() => $( $z ),*)>
    );
    ($a:ty, $( $b:ty ),+  => $( $z:ty ),* ) => (
        GeneralizedHashTrie::<var_type!( $a, $( $b ),+ ), var_type!($( $z ),*), $a, GhtNodeType!($( $b ),+ => $( $z ),*)>
    );
}
// ($a:ty => $z:tt ) => (
//     GeneralizedHashTrie::<var_type!($a), var_type!($z), $a, GhtNodeType!($z)>
// );

////////////////////////////
/// Map Compatibility Traits
////////////////////////////

/// new: (A, B, C) => (D, E, F)
/// get: (A, B, C, D) xxx
///
/// Prefix: (A, B)
/// Suffix: (C, D, E, F)
// struct HashTrieMap<Ght, Prefix, Suffix>
// where
//     Ght: GeneralizedHashTrie<Schema = var_type!(...Prefix, ...Suffix)>,
//     Prefix: VariadicExt,
//     Suffix: VariadicExt,
// {
//     data: Ght,
//     _phantom: std::marker::PhantomData<(Prefix, Suffix)>,
// }

// impl<Ght, Prefix, Suffix> HashTrieMap<Ght, Prefix, Suffix>
// where
//     Ght: GeneralizedHashTrie<Schema = var_type!(...Prefix, ...Suffix)>,
//     Prefix: VariadicExt,
//     Suffix: VariadicExt,
// {
//     pub fn new() -> Self {
//         HashTrieMap {
//             data: Default::default(),
//             _phantom: std::marker::PhantomData,
//         }
//     }

//     pub fn insert(&mut self, head: Prefix, value: Suffix) {
//         self.data.insert(var_expr!(...head, ...value));
//         // return Some(value);
//     }

//     pub fn get<'a>(&'a self, head: &'a Prefix) -> Option<Suffix::AsRefVar<'_>>
//     where
//         Ght: HtPrefixIter<Prefix::AsRefVar<'a>, Suffix = Suffix>,
//     {
//         self.data.prefix_iter(head.as_ref_var()).next()
//     }
// }

// impl<Ght, Prefix, Suffix> cc_traits::Collection for HashTrieMap<Ght, Prefix, Suffix>
// where
//     Ght: GeneralizedHashTrie<Schema = var_type!(...Prefix, ...Suffix)>,
//     Prefix: VariadicExt,
//     Suffix: VariadicExt,
// {
//     type Item = Suffix;
// }
// impl<Ght, Prefix, Suffix> cc_traits::CollectionRef for HashTrieMap<Ght, Prefix, Suffix>
// where
//     Ght: GeneralizedHashTrie<Schema = var_type!(...Prefix, ...Suffix)>,
//     Prefix: VariadicExt,
//     Suffix: VariadicExt + Clone,
// {
//     type ItemRef<'a> = Box<Suffix>
//     where
//         Self: 'a;

//     fn upcast_item_ref<'short, 'long: 'short>(r: Self::ItemRef<'long>) -> Self::ItemRef<'short>
//     where
//         Self: 'long,
//     {
//         r
//     }
// }
// impl<'k, Ght, Prefix, Suffix> cc_traits::Get<&'k Prefix> for HashTrieMap<Ght, Prefix, Suffix>
// where
//     Ght: GeneralizedHashTrie<Schema = var_type!(...Prefix, ...Suffix)>,
//     Ght: HtPrefixIter<Prefix::AsRefVar<'k>, Suffix = Suffix>,
//     Prefix: VariadicExt,
//     Suffix: 'k + VariadicExt + Clone,
//     Suffix::AsRefVar<'k>: UnrefCloneVariadic<Unref = Suffix>,
//     Self: 'k,
// {
//     fn get(&self, head: &'k Prefix) -> Option<Box<Suffix>> {
//         self.data
//             .prefix_iter(head.as_ref_var())
//             .next()
//             .map(|row| row.clone_var())
//             .map(Box::new)
//     }
// }

// impl<K, V> Default for HashTrieMap<K, V>
// where
//     K: Eq + Hash + VariadicExt + 'static,
//     V: Clone + Eq + Hash + VariadicExt + 'static,
// {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl<K, V> FromIterator<(K, V)> for HashTrieMap<K, V>
// where
//     K: Eq + Hash + VariadicExt + 'static,
//     V: Clone + Eq + Hash + VariadicExt + 'static,
// {
//     fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
//         let mut map = Self::new();
//         for (k, v) in iter {
//             map.insert(k, v);
//         }
//         map
//     }
// }

// impl<K, V> IntoIterator for HashTrieMap<K, V>
// where
//     K: Eq + Hash + VariadicExt + 'static,
//     V: Clone + Eq + Hash + VariadicExt + 'static,
// {
//     type Item = (K, V);
//     type IntoIter = std::vec::IntoIter<Self::Item>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.data.recursive_iter()
//     }
// }

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
pub struct GhtKeyedBimorphism<Bimorphism> {
    bimorphism: Bimorphism,
    // _phantom: std::marker::PhantomData<fn() -> MapOut>,
}
impl<Bimorphism> GhtKeyedBimorphism<Bimorphism> {
    /// Create a `KeyedBimorphism` using `bimorphism` for handling values.
    pub fn new(bimorphism: Bimorphism) -> Self {
        Self {
            bimorphism,
            // _phantom: std::marker::PhantomData,
        }
    }
}
// impl<Head, GhtA, GhtB, ValFunc> LatticeBimorphism<GhtA, GhtB> for GhtKeyedBimorphism<ValFunc>
// where
//     Head: Clone + Hash + Eq,
//     ValFunc: LatticeBimorphism<GhtA::Get, GhtB::Get>,
//     ValFunc::Output: GeneralizedHashTrie,
//     GhtA::Get: Clone, // BAD: cloning subtrees
//     GhtB::Get: Clone, // BAD: cloning subtrees
//     // MapA: MapIter + SimpleKeyedRef + SimpleCollectionRef,
//     // MapB: for<'a> GetKeyValue<&'a MapA::Key, Key = MapA::Key> + SimpleCollectionRef,
//     // MapA::Key: Clone + Eq,
//     // MapA::Item: Clone,
//     // MapB::Item: Clone,
//     // MapOut: Default + MapInsert<MapA::Key> + Collection<Item = ValFunc::Output>,
//     GhtA: GeneralizedHashTrie<Head = Head>,
//     GhtB: GeneralizedHashTrie<Head = Head>,
//     // GhtOut: FromIterator<(Head, ValFunc::Output)>,
//     for<'a> <GhtA::Schema as VariadicExt>::AsRefVar<'a>: UnrefCloneVariadic,
//     for<'a> <GhtB::Schema as VariadicExt>::AsRefVar<'a>: UnrefCloneVariadic,
// {
//     type Output = HtInner<Head, ValFunc::Output>; // HashMap<Head, ValFunc::Output>; // GhtOut;

//     fn call(&mut self, ght_a: GhtA, ght_b: GhtB) -> Self::Output {
//         let mut children = HashMap::<Head, ValFunc::Output>::new();
//         for head in ght_b.iter() {
//             if let Some(get_a) = ght_a.get(head) {
//                 let get_b = ght_b.get(head).unwrap();
//                 // BAD: clones subtrees
//                 let val = self.bimorphism.call(get_a.clone(), get_b.clone());
//                 children.insert(head.clone(), val);
//             }
//         }
//         HtInner { children }
//     }
// }
impl<'a, 'b, Head, GhtA, GhtB, ValFunc> LatticeBimorphism<&'a GhtA, &'b GhtB>
    for GhtKeyedBimorphism<ValFunc>
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
    type Output = HtInner<Head, ValFunc::Output>; // HashMap<Head, ValFunc::Output>; // GhtOut;

    fn call(&mut self, ght_a: &'a GhtA, ght_b: &'b GhtB) -> Self::Output {
        let mut children = HashMap::<Head, ValFunc::Output>::new();
        for head in ght_b.iter() {
            if let Some(get_a) = ght_a.get(head) {
                let get_b = ght_b.get(head).unwrap();
                let val = self.bimorphism.call(get_a, get_b);
                children.insert(head.clone(), val);
            }
        }
        HtInner { children }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NaiveLatticeOrd;

    #[test]
    fn basic_test() {
        // Example usage
        type MyTrie1 = Ght!(u32, u32 => &'static str);
        let htrie1 = MyTrie1::new_from(vec![var_expr!(42, 314, "hello")]);
        assert!(htrie1.contains(var_expr!(&42, &314, &"hello")));
        assert_eq!(htrie1.recursive_iter().count(), 1);

        type MyTrie2 = Ght!(u32 => u32);
        let htrie2 = MyTrie2::new_from(vec![var_expr!(42, 314)]);
        assert!(htrie2.contains(var_expr!(&42, &314)));
        assert_eq!(htrie1.recursive_iter().count(), 1);
    }

    #[test]
    fn test_ght_node_type_macro() {
        type LilTrie = GhtNodeType!(u32);
        let _l = LilTrie::new_from(vec![var_expr!(1)]);

        type SmallTrie = GhtNodeType!(u32, &'static str);
        type SmallKeyedTrie = GhtNodeType!(u32 => &'static str);
        let l = SmallTrie::new_from(vec![var_expr!(1, "hello")]);
        let _: SmallKeyedTrie = l;

        type LongKeySmallValTrie = GhtNodeType!(u32, u16, &'static str);
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
        let mut htrie = <Ght!(u16, u32 => u64)>::default();
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

        type LongKeyLongValTrie = Ght!(u32, u64 => u16, &'static str);
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
        type MyGht = Ght!(u16, u32 => u64);
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
        type MyGht = Ght!(u32, u32 => u32);
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
        type MyGht = Ght!(u32, u32 => u32);
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
        type MyGht = Ght!(u32, u32 => u32);
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
        type MyGht = Ght!(u32, u32 => u32);
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
        type MyGht = Ght!(bool, u32, &'static str => i32);
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
        type MyGht = Ght!(u32, u64 => u16, &'static str);

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
    fn test_lattice() {
        type MyGht = Ght!(u32, u64 => u16, &'static str);
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

        for ght in [
            empty_ght.trie,
            test_ght1.trie,
            test_ght2.trie,
            test_ght3.trie,
            test_ght4.trie,
        ] {
            ght.naive_cmp(&ght.clone());
            test_vec.push(ght);
        }
        crate::test::check_lattice_ord(&test_vec);
        crate::test::check_partial_ord_properties(&test_vec);
        crate::test::check_lattice_properties(&test_vec);
    }

    #[test]
    fn test_bimorphism() {
        type MyGhtA = Ght!(u32, u64 => u16, &'static str);
        type MyGhtB = Ght!(u32, u64, u16 => &'static str);

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
    fn test_keyed_bimorphism() {
        type MyGhtA = Ght!(u32, u64, u16 => &'static str);
        type MyGhtB = Ght!(u32, u64, u16 => &'static str);

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
            type MyGhtOut = GhtNodeType!(u64, u16, &'static str, u64, u16 => &'static str);

            let mut bim =
                GhtKeyedBimorphism::new(GhtCartesianProductBimorphism::<MyGhtOut>::default());
            let out = bim.call(&ght_a.trie, &ght_b.trie);
            println!("{:?}", out);
            for row in out.recursive_iter() {
                println!("ROW {:?}", row);
            }
        }

        {
            // type MyGhtOut = Vec<var_type!(u16, &'static str, u16, &'static str)>;
            type MyGhtOut = GhtNodeType!(u16, &'static str, u16, &'static str);
            let mut bim =
                GhtKeyedBimorphism::new(GhtKeyedBimorphism::new(GhtCartesianProductBimorphism::<
                    MyGhtOut,
                >::default()));
            let out = bim.call(&ght_a.trie, &ght_b.trie);
            println!("{:?}", out);
            for row in out.recursive_iter() {
                println!("ROW {:?}", row);
            }
        }

        {
            type MyGhtOut = GhtNodeType!(&'static str, &'static str);
            let mut bim = GhtKeyedBimorphism::new(GhtKeyedBimorphism::new(
                GhtKeyedBimorphism::new(GhtCartesianProductBimorphism::<MyGhtOut>::default()),
            ));
            let out = bim.call(&ght_a.trie, &ght_b.trie);
            println!("{:?}", out);
            for row in out.recursive_iter() {
                println!("ROW {:?}", row);
            }
        }

        // println!(
        //     "{:?}: count {:?}",
        //     ght_out,
        //     ght_out.recursive_iter().count()
        // );
    }

    #[test]
    fn test_recursive_iter_keys() {
        type MyGht = Ght!(u32 => u32, u32);
        let mut htrie = MyGht::new_from(vec![var_expr!(42, 314, 43770)]);
        htrie.insert(var_expr!(42, 315, 43770));
        htrie.insert(var_expr!(43, 10, 600));
        htrie.insert(var_expr!(43, 10, 60));
        for row in htrie.recursive_iter_keys() {
            println!("row: {:?}, height: {}", row, htrie.height().unwrap());
        }
    }

    // pub trait KeyedVariadic: Variadic {
    //     fn eq_keys(&self, other: &Self) -> bool;
    // }
    // impl<Item1, Item2, Rest> KeyedVariadic for (Item1, (Item2, Rest))
    // where
    //     Item1: PartialEq,
    //     Item2: PartialEq,
    //     Rest: VariadicExt,
    // {
    //     fn eq_keys(&self, other: &Self) -> bool {
    //         let (item1, (item2, _)) = self;
    //         let (other_item1, (other_item2, _)) = other;
    //         item1 == other_item1 && item2 == other_item2
    //     }
    // }
    // impl KeyedVariadic for () {
    //     fn eq_keys(&self, _other: &Self) -> bool {
    //         true
    //     }
    // }

    // #[macro_export]
    // macro_rules! KeyedVariadicEq {
    //     // Base case: when the variadic list is empty, return true.
    //     () => {
    //         true
    //     };
    //     // Recursive case: when the last element is a nested singleton variadic.
    //     ($a:expr, ($b:expr, ())) => {
    //         $a
    //     };
    //     // Recursive case: when there are more elements in the variadic list.
    //     ($a:expr, $($rest:tt)*) => {
    //         $a && KeyedVariadicEq!($($rest)*)
    //     };
    // }

    // // Define an enum to encapsulate the tuple variations
    // enum TupleVariant<T, U>
    // where
    //     U: VariadicExt,
    // {
    //     GeneralCase(T, U),
    //     UnitCase(T),
    // }
    // // Implement the trait for the enum
    // impl<Head, Rest> PartialEqExceptNested for TupleVariant<Head, Rest>
    // where
    //     TupleVariant<Head, Rest>: VariadicExt,
    //     Head: PartialEq,
    //     Rest: VariadicExt + PartialEqExceptNested,
    // {
    //     fn eq_except_nested(&self, other: &Self) -> bool {
    //         match self {
    //             TupleVariant::GeneralCase(head, rest) => {
    //                 if let TupleVariant::GeneralCase(other_item, other_rest) = other {
    //                     head == other_item && rest.eq_except_nested(other_rest)
    //                 } else {
    //                     panic!()
    //                 }
    //             }
    //             TupleVariant::UnitCase(_head) => true,
    //         }
    //     }
    // }

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

    //     #[test]
    //     fn test_key_cmp() {
    //         type KeyType = var_type!(u16, u32);
    //         type ValType = var_type!(u64);
    //         type ChildGht = GhtType!(u32 => u64);
    //         type HeadType = u16;
    //         type MyRoot = GhtRoot<KeyType, ValType, HeadType, ChildGht>;
    //         let mut trie1: MyRoot = GhtRoot::default();
    //         trie1.trie.insert(var_expr!(1, 2, 3));
    //         let mut trie2: MyRoot = GhtRoot::default();
    //         trie2.trie.insert(var_expr!(1, 2, 4));

    //         let tup1 = trie1.trie.recursive_iter_keys().next().unwrap();
    //         let tup2 = trie2.trie.recursive_iter_keys().next().unwrap();
    //         assert!(tup1 != tup2);
    //         let t = trie1.tree;
    //         let key1 = trie1.trie::Schema::<MyRoot::KeyType>(tup1);
    //         let key2 = tup2.split(MyRoot::KeyType);
    //         assert_eq!(key1, key2);
    //         println!("key1 is {:?}, key2 is {:?}", key1, key2);
    //     }
}
