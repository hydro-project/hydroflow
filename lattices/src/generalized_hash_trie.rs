use std::cmp::Ordering::{self, *};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use sealed::sealed;
use variadics::{var_args, var_expr, var_type, AsRefVariadicPartialEq, Variadic, VariadicExt};

use crate::{IsBot, IsTop, LatticeOrd, Merge};

/// GeneralizedHashTrie node trait
#[sealed]
pub trait GeneralizedHashTrie: Default {
    /// Schema variadic: the type of rows we're storing
    type Schema: VariadicExt + AsRefVariadicPartialEq;
    /// The type of the first column in the Schema
    type Key: Eq + Hash;

    /// Create a new, empty GHT
    fn new(input: Vec<Self::Schema>) -> Self;

    /// Inserts items into the hash trie.
    fn insert(&mut self, row: Self::Schema) -> bool;

    /// Returns `true` if the (entire) row is found in the trie, `false` otherwise.
    /// See `get()` below to look just for keys in this node
    fn contains(&self, row: <Self::Schema as VariadicExt>::AsRefVar<'_>) -> bool;

    /// Iterator for the keys (from inner nodes) or elements (from leaf nodes).
    fn iter(&self) -> impl Iterator<Item = &'_ Self::Key>;

    /// Type returned by [`Self::get`].
    type Get: GeneralizedHashTrie;

    /// On an Inner node, retrieves the value (child) associated with the given key.
    /// returns an `Option` containing a reference to the value if found, or `None` if not found.
    /// On a Leaf node, returns None.
    fn get(&self, key: &Self::Key) -> Option<&'_ Self::Get>;

    /// Iterate through the (entire) rows stored in this HashTrie.
    /// Returns Variadics, not tuples.
    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>>;
}

/// internal node of a HashTrie
#[derive(Debug, Clone)]
pub struct HtInner<Key, Node>
where
    Node: GeneralizedHashTrie,
{
    children: HashMap<Key, Node>,
}
impl<Key, Node: GeneralizedHashTrie> Default for HtInner<Key, Node>
where
    Node: GeneralizedHashTrie,
{
    fn default() -> Self {
        let children = Default::default();
        Self { children }
    }
}
#[sealed]
impl<Key, Node> GeneralizedHashTrie for HtInner<Key, Node>
where
    Key: 'static + Hash + Eq,
    Node: 'static + GeneralizedHashTrie + Merge<Node>,
{
    type Schema = var_type!(Key, ...Node::Schema);
    type Key = Key;

    fn new(input: Vec<Self::Schema>) -> Self {
        let mut retval: Self = Default::default();
        for i in input {
            retval.insert(i);
        }
        retval
    }

    fn insert(&mut self, row: Self::Schema) -> bool {
        let var_args!(key, ...rest) = row;
        self.children.entry(key).or_default().insert(rest)
    }

    fn contains(&self, row: <Self::Schema as VariadicExt>::AsRefVar<'_>) -> bool {
        let var_args!(key, ...rest) = row;
        if let Some(node) = self.children.get(key) {
            node.contains(rest)
        } else {
            false
        }
    }

    fn iter(&self) -> impl Iterator<Item = &'_ Self::Key> {
        self.children.keys()
    }

    type Get = Node;
    fn get(&self, key: &Self::Key) -> Option<&'_ Self::Get> {
        self.children.get(key)
    }

    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
        self.children
            .iter()
            .flat_map(|(k, vs)| vs.recursive_iter().map(move |v| var_expr!(k, ...v)))
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
impl<T> GeneralizedHashTrie for HtLeaf<T>
where
    T: 'static + Eq + VariadicExt + AsRefVariadicPartialEq + Hash,
{
    type Schema = T;
    type Key = T;

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

    fn iter(&self) -> impl Iterator<Item = &'_ Self::Key> {
        self.elements.iter()
    }

    type Get = Self;
    fn get(&self, _key: &Self::Key) -> Option<&'_ Self::Get> {
        Option::<&Self>::None
    }

    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Schema as VariadicExt>::AsRefVar<'_>> {
        self.elements.iter().map(T::as_ref_var)
    }
}

// impl<Key, Node> Len for HtInner<Key, Node>
// where
//     Node: GeneralizedHashTrie,
// {
//     fn len(&self) -> usize {
//         self.len
//     }
// }

// impl<T> Len for HtLeaf<T>
// where
//     T: Hash + Eq,
// {
//     fn len(&self) -> usize {
//         self.elements.len()
//     }
// }

impl<Key, Node> Merge<HtInner<Key, Node>> for HtInner<Key, Node>
where
    Node: GeneralizedHashTrie + Merge<Node>,
    Self: GeneralizedHashTrie,
    Key: Hash + Eq,
{
    fn merge(&mut self, other: HtInner<Key, Node>) -> bool {
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

impl<Key, Node> PartialEq<HtInner<Key, Node>> for HtInner<Key, Node>
where
    Key: Hash + Eq + 'static,
    Node: GeneralizedHashTrie + 'static + PartialEq + Merge<Node>,
{
    fn eq(&self, other: &HtInner<Key, Node>) -> bool {
        if self.children.len() != other.children.len() {
            return false;
        }

        for key in self.iter() {
            if other.get(key).is_none() {
                return false;
            }
            if !self.get(key).unwrap().eq(other.get(key).unwrap()) {
                return false;
            }
        }
        true
    }
}

impl<Key, Node> PartialOrd<HtInner<Key, Node>> for HtInner<Key, Node>
where
    Key: Hash + Eq + 'static,
    Node: GeneralizedHashTrie + PartialEq + Merge<Node> + 'static + PartialOrd,
{
    fn partial_cmp(&self, other: &HtInner<Key, Node>) -> Option<Ordering> {
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
                if other.elements.iter().all(|key| self.elements.contains(key)) {
                    Some(Greater)
                } else {
                    None
                }
            }
            Equal => {
                if self.elements.iter().all(|key| other.elements.contains(key)) {
                    Some(Equal)
                } else {
                    None
                }
            }
            Less => {
                if self.elements.iter().all(|key| other.elements.contains(key)) {
                    Some(Less)
                } else {
                    None
                }
            }
        }
    }
}

impl<Key, Node> LatticeOrd<HtInner<Key, Node>> for HtInner<Key, Node>
where
    Self: PartialOrd<HtInner<Key, Node>>,
    Node: GeneralizedHashTrie,
{
}
impl<T> LatticeOrd<HtLeaf<T>> for HtLeaf<T>
where
    Self: PartialOrd<HtLeaf<T>>,
    T: Hash + Eq,
{
}

impl<Key, Node> Eq for HtInner<Key, Node>
where
    Node: GeneralizedHashTrie,
    Self: PartialEq,
{
}

impl<Key, Node> IsBot for HtInner<Key, Node>
where
    Node: GeneralizedHashTrie + IsBot,
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

impl<Key, Node> IsTop for HtInner<Key, Node>
where
    Node: GeneralizedHashTrie,
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
impl<'k, Key, Node, PrefixRest> HtPrefixIter<var_type!(&'k Key, ...PrefixRest)>
    for HtInner<Key, Node>
where
    Key: Eq + Hash,
    Node: GeneralizedHashTrie + HtPrefixIter<PrefixRest>,
    Node::Suffix: 'static,      // meh
    PrefixRest: 'static + Copy, // meh
{
    type Suffix = Node::Suffix;
    fn prefix_iter<'a>(
        &'a self,
        prefix: var_type!(&'k Key, ...PrefixRest),
    ) -> impl Iterator<Item = <Self::Suffix as VariadicExt>::AsRefVar<'a>>
    where
        Self::Suffix: 'a,
    {
        let var_args!(key, ...rest) = prefix;
        self.children
            .get(key)
            .map(|node| node.prefix_iter(rest))
            .into_iter()
            .flatten()
    }
}
#[sealed]
impl<'k, Key> HtPrefixIter<var_type!(&'k Key)> for HtLeaf<Key>
where
    Key: Eq + Hash,
{
    type Suffix = var_expr!();
    fn prefix_iter<'a>(
        &'a self,
        prefix: var_type!(&'k Key),
    ) -> impl Iterator<Item = <Self::Suffix as VariadicExt>::AsRefVar<'a>>
    where
        Self::Suffix: 'a,
    {
        let var_args!(key) = prefix;
        self.elements.contains(key).then_some(()).into_iter()
    }
}
#[sealed]
impl<This> HtPrefixIter<var_type!()> for This
where
    This: 'static + GeneralizedHashTrie,
{
    type Suffix = <Self as GeneralizedHashTrie>::Schema;
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
macro_rules! GHTType {
    // Flat base case.
    ($a:ty) => {
        HtLeaf::<var_type!($a)>
    };
    // Flat recursive case.
    ($a:ty, $( $b:ty ),* ) => (
        HtInner::<$a, GHTType!($( $b ),*)>
    );

    // Arrow base case.
    ($a:ty => $( $z:ty ),*) => (
        HtInner::<$a, HtLeaf::<var_type!($( $z ),*)>>
    );
    // Arrow recursive case.
    ($a:ty, $( $b:ty ),* => $( $z:ty ),*) => (
        HtInner::<$a, GHTType!($( $b ),* => $( $z ),*)>
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NaiveLatticeOrd;

    #[test]
    fn basic_test() {
        // Example usage
        type MyTrie1 = GHTType!(u32, u32, &'static str);
        let htrie1 = MyTrie1::new(vec![var_expr!(42, 314, "hello")]);
        assert!(htrie1.contains(var_expr!(&42, &314, &"hello")));
        assert_eq!(htrie1.recursive_iter().count(), 1);

        type MyTrie2 = GHTType!(u32, u32);
        let htrie2 = MyTrie2::new(vec![var_expr!(42, 314)]);
        assert!(htrie2.contains(var_expr!(&42, &314)));
        assert_eq!(htrie1.recursive_iter().count(), 1);

        type MyTrie3 = GHTType!(&'static str);
        let htrie3 = MyTrie3::new(vec![var_expr!("Rust")]);
        assert!(htrie3.contains(var_expr!(&"Rust")));
        assert_eq!(htrie1.recursive_iter().count(), 1);
    }

    #[test]
    fn test_hash_trie_type_macro() {
        type LilTrie = GHTType!(u32);
        let _l = LilTrie::new(vec![var_expr!(1)]);

        type SmallTrie = GHTType!(u32, &'static str);
        type SmallKeyedTrie = GHTType!(u32 => &'static str);
        let l = SmallTrie::new(vec![var_expr!(1, "hello")]);
        let _: SmallKeyedTrie = l;

        type LongKeySmallValTrie = GHTType!(u32, u16, &'static str);
        type LongKeySmallValKeyedTrie = GHTType!(u32, u16 => &'static str);
        let x = LongKeySmallValTrie::new(vec![var_expr!(1, 314, "hello")]);
        let _: LongKeySmallValKeyedTrie = x;
        let _ = LongKeySmallValTrie::new(vec![var_expr!(1, 314, "hello")]);

        type SmallKeyLongValTrie = GHTType!(u32 => u64, u16, &'static str);
        // type SmallKeyLongValVTrie =
        //     GeneralizedHashTrieVariadic!(var_type!(u32 => var_type!(u64, u16, &'static str)));
        let _x = SmallKeyLongValTrie::new(vec![var_expr!(1, 999, 222, "hello")]);
        // let _: SmallKeyLongValVTrie = x;

        type LongKeyLongValTrie = GHTType!(u32, u64 => u16, &'static str);
        // type LongKeyLongValVTrie =
        //     GeneralizedHashTrieVariadic!(var_type!(u32, u64 => var_type!(u16, &'static str)));
        let _x = LongKeyLongValTrie::new(vec![var_expr!(1, 999, 222, "hello")]);
        // let _: LongKeyLongValVTrie = x;
    }

    #[test]
    fn test_insert() {
        let mut htrie = <GHTType!(u16, u32, u64)>::new(vec![]);
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

        type LongKeyLongValTrie = GHTType!(u32, u64 => u16, &'static str);
        let mut htrie = LongKeyLongValTrie::new(vec![var_expr!(1, 999, 222, "hello")]);
        htrie.insert(var_expr!(1, 999, 111, "bye"));
        htrie.insert(var_expr!(1, 1000, 123, "cya"));
        println!("htrie: {:?}", htrie);
        assert!(htrie.contains(var_expr!(&1, &999, &222, &"hello")));
        assert!(htrie.contains(var_expr!(&1, &999, &111, &"bye")));
        assert!(htrie.contains(var_expr!(&1, &1000, &123, &"cya")));
    }

    #[test]
    fn test_scale() {
        type MyGHT = GHTType!(bool, usize, &'static str, i32);
        let mut htrie = MyGHT::new(vec![var_expr!(true, 1, "hello", -5)]);
        assert_eq!(htrie.recursive_iter().count(), 1);
        for i in 1..1000000 {
            htrie.insert(var_expr!(true, 1, "hello", i));
        }
        assert_eq!(htrie.recursive_iter().count(), 1000000);
    }

    #[test]
    fn test_contains() {
        type MyGHT = GHTType!(u16, u32, u64);
        let htrie = MyGHT::new(vec![var_expr!(42_u16, 314_u32, 43770_u64)]);
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
        type MyGHT = GHTType!(u32, u32, u32);
        let mut htrie = MyGHT::new(vec![var_expr!(42, 314, 43770)]);
        htrie.insert(var_expr!(42, 315, 43770));
        htrie.insert(var_expr!(42, 314, 30619));
        htrie.insert(var_expr!(43, 10, 600));
        for row in htrie.recursive_iter() {
            println!("{:?}", row);
        }
    }

    #[test]
    fn test_get() {
        type MyGHT = GHTType!(u32, u32, u32);
        let ht_root = MyGHT::new(vec![var_expr!(42, 314, 43770)]);

        let inner = ht_root.get(&42).unwrap();
        let t = inner.recursive_iter().next().unwrap();
        assert_eq!(t, var_expr!(&314, &43770));

        let leaf = inner.get(&314).unwrap();
        let t = leaf.recursive_iter().next().unwrap();
        assert_eq!(t, var_expr!(&43770));
    }

    #[test]
    fn test_iter() {
        type MyGHT = GHTType!(u32, u32, u32);
        let ht_root = MyGHT::new(vec![var_expr!(42, 314, 43770)]);
        let inner_key = ht_root.iter().next().unwrap();
        let inner = ht_root.get(inner_key).unwrap();
        let t = inner.recursive_iter().next().unwrap();
        assert_eq!(t, var_expr!(&314, &43770));

        let leaf_key = inner.iter().next().unwrap();
        let leaf = inner.get(leaf_key).unwrap();
        let t = leaf.iter().next().unwrap();
        assert_eq!(t, &var_expr!(43770));
    }

    #[test]
    fn test_prefix_iter() {
        type MyGHT = GHTType!(u32, u32, u32);
        let mut htrie = MyGHT::new(vec![var_expr!(42, 314, 43770)]);

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
        type MyGHT = GHTType!(bool, u32, &'static str, i32);
        let mut htrie = MyGHT::new(vec![var_expr!(true, 1, "hello", -5)]);

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
        type MyGHT = GHTType!(u32, u64 => u16, &'static str);

        let mut test_ght1 = MyGHT::new(vec![var_expr!(42, 314, 10, "hello")]);
        let mut test_ght2 = MyGHT::new(vec![var_expr!(42, 314, 10, "hello")]);

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
        type MyGHT = GHTType!(u32, u64 => u16, &'static str);

        let mut test_vec: Vec<MyGHT> = Vec::new();

        let empty_ght = MyGHT::new(vec![]);
        let test_ght1 = MyGHT::new(vec![var_expr!(42, 314, 10, "hello")]);
        let mut test_ght2 = MyGHT::new(vec![var_expr!(42, 314, 10, "hello")]);
        test_ght2.insert(var_expr!(42, 314, 20, "again"));
        let mut test_ght3 = test_ght2.clone();
        test_ght3.insert(var_expr!(42, 400, 1, "level 2"));
        let mut test_ght4 = test_ght3.clone();
        test_ght4.insert(var_expr!(43, 1, 1, "level 1"));

        for ght in [empty_ght, test_ght1, test_ght2, test_ght3, test_ght4] {
            ght.naive_cmp(&ght.clone());
            test_vec.push(ght);
        }
        crate::test::check_lattice_ord(&test_vec);
        crate::test::check_partial_ord_properties(&test_vec);
        crate::test::check_lattice_properties(&test_vec);
    }
}
