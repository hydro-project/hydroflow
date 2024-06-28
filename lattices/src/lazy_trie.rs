use std::cmp::Ordering::{self, *};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use sealed::sealed;
use variadics::{
    var_args, var_expr, var_type, AsRefVariadicPartialEq, UnrefCloneVariadic, Variadic, VariadicExt,
};

use crate::{IsBot, IsTop, LatticeOrd, Merge};

/// LazyTrie node trait
#[sealed]
pub trait LazyTrie: Default {
    /// Schema variadic: the type of rows we're storing
    type Schema: VariadicExt + AsRefVariadicPartialEq;
    /// The type of the first column in the Schema
    type Head: Eq + Hash;

    /// Create a new, empty GHT
    fn new(input: Vec<Self::Schema>) -> Self;

    /// Inserts items into the hash trie.
    fn insert(&mut self, row: Self::Schema) -> bool;

    /// Returns `true` if the (entire) row is found in the trie, `false` otherwise.
    /// See `get()` below to look just for "head" keys in this node
    fn contains(&self, row: <Self::Schema as VariadicExt>::AsRefVar<'_>) -> bool;

    /// Iterator for the "head" keys (from inner nodes) or elements (from leaf nodes).
    fn iter(&self) -> impl Iterator<Item = &'_ Self::Head>;

    /// Type returned by [`Self::get`].
    type Get: LazyTrie;

    /// On an Inner node, retrieves the value (child) associated with the given "head" key.
    /// returns an `Option` containing a reference to the value if found, or `None` if not found.
    /// On a Leaf node, returns None.
    fn get(&self, head: &Self::Head) -> Option<&'_ Self::Get>;

    /// Iterate through the (entire) rows stored in this HashTrie.
    /// Returns Variadics, not tuples.
    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>>;
}

/// internal node of a LazyTrie
#[derive(Debug, Clone)]
pub struct LtInner<Head, Node>
where
    Node: LazyTrie,
{
    children: HashMap<Head, Node>,
}
impl<Head, Node: LazyTrie> Default for LtInner<Head, Node>
where
    Node: LazyTrie,
{
    fn default() -> Self {
        let children = Default::default();
        Self { children }
    }
}
#[sealed]
impl<Head, Node> LazyTrie for LtInner<Head, Node>
where
    Head: 'static + Hash + Eq,
    Node: 'static + LazyTrie + Merge<Node>,
{
    type Schema = var_type!(Head, ...Node::Schema);
    type Head = Head;

    fn new(input: Vec<Self::Schema>) -> Self {
        let mut retval: Self = Default::default();
        for i in input {
            retval.insert(i);
        }
        retval
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
}

/// leaf node of a HashTrie
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LtLeaf<T>
where
    T: Hash + Eq,
{
    elements: HashSet<T>,
}
impl<T> Default for LtLeaf<T>
where
    T: Hash + Eq,
{
    fn default() -> Self {
        let elements = Default::default();
        Self { elements }
    }
}
#[sealed]
impl<T> LazyTrie for LtLeaf<T>
where
    T: 'static + Eq + VariadicExt + AsRefVariadicPartialEq + Hash,
{
    type Schema = T;
    type Head = T;

    fn new(input: Vec<Self::Schema>) -> Self {
        let mut retval: Self = Default::default();
        for i in input {
            retval.insert(i);
        }
        retval
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
}

impl<Head, Node> Merge<LtInner<Head, Node>> for LtInner<Head, Node>
where
    Node: LazyTrie + Merge<Node>,
    Self: LazyTrie,
    Head: Hash + Eq,
{
    fn merge(&mut self, other: LtInner<Head, Node>) -> bool {
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

impl<T> Merge<LtLeaf<T>> for LtLeaf<T>
where
    T: Hash + Eq,
{
    fn merge(&mut self, other: LtLeaf<T>) -> bool {
        let old_len = self.elements.len();
        self.elements.extend(other.elements);
        self.elements.len() > old_len
    }
}

impl<Head, Node> PartialEq<LtInner<Head, Node>> for LtInner<Head, Node>
where
    Head: Hash + Eq + 'static,
    Node: LazyTrie + 'static + PartialEq + Merge<Node>,
{
    fn eq(&self, other: &LtInner<Head, Node>) -> bool {
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

impl<Head, Node> PartialOrd<LtInner<Head, Node>> for LtInner<Head, Node>
where
    Head: Hash + Eq + 'static,
    Node: LazyTrie + PartialEq + Merge<Node> + 'static + PartialOrd,
{
    fn partial_cmp(&self, other: &LtInner<Head, Node>) -> Option<Ordering> {
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

impl<T> PartialOrd<LtLeaf<T>> for LtLeaf<T>
where
    T: Hash + Eq + 'static,
{
    fn partial_cmp(&self, other: &LtLeaf<T>) -> Option<Ordering> {
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

impl<Head, Node> LatticeOrd<LtInner<Head, Node>> for LtInner<Head, Node>
where
    Self: PartialOrd<LtInner<Head, Node>>,
    Node: LazyTrie,
{
}
impl<T> LatticeOrd<LtLeaf<T>> for LtLeaf<T>
where
    Self: PartialOrd<LtLeaf<T>>,
    T: Hash + Eq,
{
}

impl<Head, Node> Eq for LtInner<Head, Node>
where
    Node: LazyTrie,
    Self: PartialEq,
{
}

impl<Head, Node> IsBot for LtInner<Head, Node>
where
    Node: LazyTrie + IsBot,
{
    fn is_bot(&self) -> bool {
        self.children.iter().all(|(_, v)| v.is_bot())
    }
}

impl<T> IsBot for LtLeaf<T>
where
    T: Hash + Eq,
{
    fn is_bot(&self) -> bool {
        self.elements.is_empty()
    }
}

impl<Head, Node> IsTop for LtInner<Head, Node>
where
    Node: LazyTrie,
{
    fn is_top(&self) -> bool {
        false
    }
}

impl<T> IsTop for LtLeaf<T>
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
impl<'k, Head, Node, PrefixRest> HtPrefixIter<var_type!(&'k Head, ...PrefixRest)>
    for LtInner<Head, Node>
where
    Head: Eq + Hash,
    Node: LazyTrie + HtPrefixIter<PrefixRest>,
    Node::Suffix: 'static,      // meh
    PrefixRest: 'static + Copy, // meh
{
    type Suffix = Node::Suffix;
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
impl<'k, Head> HtPrefixIter<var_type!(&'k Head)> for LtLeaf<Head>
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
    This: 'static + LazyTrie,
{
    type Suffix = <Self as LazyTrie>::Schema;
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

/// Construct a GHT type from the constituent key and
/// dependent column types. You pass it either:
///    - a flat tuple of types, a la (T, U, V)
///    - or a list of key column types and dependent column type separated by a fat arrow,
///         a la (K1, K2, K3 => T1, T2, T3)
/// This macro generates a ght where each key column is associated with a level of HtInner
/// nodes, and the remaining dependent columns are associated with a variadic HTleaf level
/// a la var_expr!(T1, T2, T3)
#[macro_export]
macro_rules! LTType {
    // Flat base case: one column.
    ($a:ty) => {
        LtLeaf::<var_type!($a)>
    };

    // Flat base case: two columns.
    ($a:ty, ( $b:ty , ()) ) => {
        LtLeaf::<var_type!($a), var_type!($b)>
    };

    // // Flat recursive case.
    // ($a:ty, $( $b:ty ),* ) => (
    //     LTType!(var_type!($a, ...$b))
    // );

    // Arrow case.
    ($a:ty => $( $z:ty ),*) => (
        LtLeaf::<$a, var_type!($( $z ),*)>
    );
    // // Arrow recursive case.
    // ($a:ty, $( $b:ty ),* => $( $z:ty ),*) => (
    //     LtLeaf::<$a, LTType!($( $b ),* => $( $z ),*)>
    // );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NaiveLatticeOrd;

    #[test]
    fn basic_test() {
        // Example usage
        type MyTrie1 = LTType!(u32);
        let htrie1 = MyTrie1::new(vec![var_expr!(42)]);
        assert!(htrie1.contains(var_expr!(&42)));

        type MyTrie2 = LTType!(u32 => u32);
        let htrie1 = MyTrie1::new(vec![var_expr!(42, 43)]);
        assert!(htrie1.contains(var_expr!(&42, &43)));

        // type MyTrie2 = LTType!(u32, u32);
        // let htrie2 = MyTrie2::new(vec![var_expr!(42, 314)]);
        // assert!(htrie2.contains(var_expr!(&42, &314)));
        // assert_eq!(htrie1.recursive_iter().count(), 1);

        // type MyTrie3 = LTType!(&'static str);
        // let htrie3 = MyTrie3::new(vec![var_expr!("Rust")]);
        // assert!(htrie3.contains(var_expr!(&"Rust")));
        // assert_eq!(htrie1.recursive_iter().count(), 1);
    }
}
