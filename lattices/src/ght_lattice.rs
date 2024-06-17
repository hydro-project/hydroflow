use core::cmp::Ordering::{Equal, Greater, Less};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

use variadics::{var_expr, var_type, AsRefVariadicPartialEq, UnrefCloneVariadic, VariadicExt};

use crate::ght::{GeneralizedHashTrie, GeneralizedHashTrieNode, GhtInner, GhtLeaf, GHT};
use crate::{IsBot, IsTop, LatticeBimorphism, LatticeOrd, Merge};

//////////////////////////
/// Lattice traits for GHT
//////////////////////////

/// Merge
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

/// PartialEq
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

// PartialOrd
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

// LatticeOrd
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

// IsBot
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

// IsTop
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

//////////////////////////
/// BiMorphisms for GHT
//////////////////////////
/// Bimorphism for the cartesian product of two GHTs. Output is a set of all possible pairs of
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

impl<GhtA, GhtB, ValFunc, Output> LatticeBimorphism<GhtA, GhtB> for GhtBimorphism<ValFunc>
where
    GhtA: GeneralizedHashTrie,
    GhtB: GeneralizedHashTrie,
    for<'a, 'b> ValFunc: LatticeBimorphism<&'a GhtA::Trie, &'b GhtB::Trie, Output = Output>,
    Output: GeneralizedHashTrieNode,
    <<GhtA as GeneralizedHashTrie>::ValType as VariadicExt>::Extend<
        <GhtB as GeneralizedHashTrie>::ValType,
    >: AsRefVariadicPartialEq,
{
    type Output = GHT<GhtA::KeyType, var_type!(...GhtA::ValType, ...GhtB::ValType), Output>; // HashMap<Head, ValFunc::Output>; // GhtOut;

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
    GhtA: GeneralizedHashTrieNode<Head = Head>,
    GhtB: GeneralizedHashTrieNode<Head = Head>,
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
