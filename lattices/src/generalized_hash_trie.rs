use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use sealed::sealed;
use variadics::{var_args, var_expr, var_type, AsRefVariadicPartialEq, Variadic, VariadicExt};

use crate::cc_traits::Len;
use crate::Merge;

// pub fn flatten_tuple(v: Variadic) -> Variadic {
//     let (head, tail) = v;

// }
/// node of a HashTrie
#[sealed]
pub trait GeneralizedHashTrie: Default {
    /// Row variadic.
    type Row: VariadicExt + AsRefVariadicPartialEq;
    /// The first item in Row.
    type Key: Eq + Hash;

    /// Inserts items into the hash trie.
    fn insert(&mut self, row: Self::Row) -> bool;

    /// Returns `true` if the row is found, `false` otherwise.
    fn contains(&self, row: <Self::Row as VariadicExt>::AsRefVar<'_>) -> bool;

    /// Iterator for the keys (inner nodes) or elements (leaf nodes).
    fn iter(&self) -> impl Iterator<Item = &'_ Self::Key>;

    /// Type returned by [`Self::get`].
    type Get: GeneralizedHashTrie;
    /// Retrieves the value associated with the given key.
    /// returns an `Option` containing a reference to the value if found, or `None` if not found.
    fn get(&self, key: &Self::Key) -> Option<&'_ Self::Get>;

    /// Iterate the sub-tree.
    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Row as VariadicExt>::AsRefVar<'_>>;
}

impl<Key, Node: GeneralizedHashTrie> Len for HtInner<Key, Node> {
    fn len(&self) -> usize {
        self.len
    }
}

/// internal node of a HashTrie
#[derive(Debug)]
pub struct HtInner<Key, Node>
where
    Node: GeneralizedHashTrie,
{
    children: HashMap<Key, Node>,
    len: usize,
}
impl<Key, Node: GeneralizedHashTrie> Default for HtInner<Key, Node>
where
    Node: GeneralizedHashTrie,
{
    fn default() -> Self {
        let children = Default::default();
        Self { children, len: 0 }
    }
}
#[sealed]
impl<Key, Node> GeneralizedHashTrie for HtInner<Key, Node>
where
    Key: 'static + Hash + Eq,
    Node: 'static + GeneralizedHashTrie,
{
    type Row = var_type!(Key, ...Node::Row);
    type Key = Key;

    fn insert(&mut self, row: Self::Row) -> bool {
        let var_args!(key, ...rest) = row;

        let retval = self.children.entry(key).or_default().insert(rest);
        if retval {
            self.len += 1;
        }
        retval
    }

    fn contains(&self, row: <Self::Row as VariadicExt>::AsRefVar<'_>) -> bool {
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

    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Row as VariadicExt>::AsRefVar<'_>> {
        self.children
            .iter()
            .flat_map(|(k, vs)| vs.recursive_iter().map(move |v| var_expr!(k, ...v)))
    }
}

// /// Iter for HTLeaf.
// pub type HtInnerIter<'a, Key, Node>
// where
//     Key: 'static + Eq + Hash,
//     Node: 'static + HashTrieNode,
// = impl 'a
//     + Iterator<Item = <<HtInner<Key, Node> as HashTrieNode>::Row as VariadicExt>::AsRefVar<'a>>;

/// leaf node of a HashTrie
#[derive(Debug, PartialEq, Eq)]
pub struct HtLeaf<T> {
    elements: Vec<T>,
}
impl<T> Default for HtLeaf<T> {
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
    type Row = T;
    type Key = T;

    fn insert(&mut self, row: Self::Row) -> bool {
        self.elements.push(row);
        true
    }

    fn contains(&self, row: <Self::Row as VariadicExt>::AsRefVar<'_>) -> bool {
        self.elements.iter().any(|r| r.as_ref_var_eq(row))
    }

    fn iter(&self) -> impl Iterator<Item = &'_ Self::Key> {
        self.elements.iter()
    }

    type Get = Self;
    fn get(&self, _key: &Self::Key) -> Option<&'_ Self::Get> {
        Option::<&Self>::None
    }

    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Row as VariadicExt>::AsRefVar<'_>> {
        self.elements.iter().map(T::as_ref_var)
    }
}

impl<T> Len for HtLeaf<T> {
    fn len(&self) -> usize {
        self.elements.len()
    }
}

impl<Key, Node> Merge<HtInner<Key, Node>> for HtInner<Key, Node>
where
    Node: GeneralizedHashTrie + Merge<Node>,
    Self: GeneralizedHashTrie,
    Key: Hash + Eq,
{
    // XX Need to revisit to make length calculation incremental/efficient
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
        // XXX inefficient
        self.len = self.recursive_iter().count();
        changed
    }
}

impl<T> Merge<HtLeaf<T>> for HtLeaf<T>
where
    T: Extend<T> + Len + IntoIterator<Item = T>,
{
    // Not worrying about dups right now
    fn merge(&mut self, other: HtLeaf<T>) -> bool {
        let old_len = self.len();
        self.elements.extend(other.elements);
        self.len() > old_len
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
    type Suffix = <Self as GeneralizedHashTrie>::Row;
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

/// Trait to convert a tuple to a HashTrieList
#[sealed]
pub trait ToHashTrie: Variadic {
    /// The output type of the hash trie conversion.
    type Output: GeneralizedHashTrie;

    /// Converts the object into a hash trie.
    fn to_hash_trie(self) -> Self::Output;
}

#[sealed]
impl<Head> ToHashTrie for var_type!(Head)
where
    Head: 'static + Eq + Hash + VariadicExt + AsRefVariadicPartialEq,
{
    type Output = HtLeaf<Head>;

    fn to_hash_trie(self) -> Self::Output {
        let mut result = HtLeaf {
            elements: Vec::new(),
        };
        result.elements.push(self.0);
        result
    }
}

#[sealed]
impl<Head, Tail> ToHashTrie for var_type!(Head, ...Tail)
where
    Head: 'static + Eq + Hash,
    Tail: 'static + ToHashTrie,
{
    type Output = HtInner<Head, Tail::Output>;

    fn to_hash_trie(self) -> Self::Output {
        let var_args!(head, ...tail) = self;
        let mut result = HtInner::<Head, Tail::Output> {
            children: HashMap::<Head, Tail::Output>::new(),
            len: 1,
        };
        result.children.insert(head, tail.to_hash_trie());
        result
    }
}

/// Construct a GHT type from the constituent key and
/// dependent column types
#[macro_export]
macro_rules! GeneralizedHashTrie {
    // Flat base case.
    ($a:ty) => {
        HtLeaf::<var_type!($a)>
    };
    // Flat recursive case.
    ($a:ty, $( $b:ty ),* ) => (
        HtInner::<$a, GeneralizedHashTrie!($( $b ),*)>
    );

    // Arrow base case.
    ($a:ty => $( $z:ty ),*) => (
        HtInner::<$a, HtLeaf::<var_type!($( $z ),*)>>
    );
    // Arrow recursive case.
    ($a:ty, $( $b:ty ),* => $( $z:ty ),*) => (
        HtInner::<$a, GeneralizedHashTrie!($( $b ),* => $( $z ),*)>
    );
}

// #[macro_export]
// macro_rules! GeneralizedHashTrieVariadic {
//     // Flat base case.
//     (var_type !($a:ty)) => {
//         HtLeaf::<var_type!($a)>
//     };
//     // Nested base case.
//     (var_type!($a:ty, var_type!($( $z:ty ),*))) => (
//         HtInner::<$a, HtLeaf::<$z>>
//     );
//     // Flat tuple base case2: when there are exactly two fields
//     (var_type !($a:ty, $b:ty)) => {
//         HtInner::<$a, HtLeaf<var_type!($b)>>
//     };
//     // Flat tuple recursive case: when there are more than two fields
//     (var_type !($a:ty, $($b:ty),*)) => {
//         HtInner::<$a, GeneralizedHashTrieVariadic!(var_type!($( $b ),*))>
//     };

//     // Arrow base case.
//     (var_type!($a:ty => $($b:ty),*)) => (
//         HtInner::<$a, HtLeaf::<$($b),*>>
//     );
//     // Arrow recursive case.
//     (var_type!($a:ty, $( $b:ty ),* => $($z:ty),*)) => (
//         HtInner::<$a, GeneralizedHashTrieVariadic!(var_type!($( $b ),* => $( $z ),*))>
//     );
// }

/// a ght tuple is a variadic for the key columns, which has a nested variadic at the
/// innermost position representing the dependent columns.
/// In a flat tuple, the innermost position is a nested variadic of the rightmost item:
/// `(item, ())`.
/// This macro generates a ght tuple from either:
///    - a flat tuple, a la (a, b, c)
///    - or key columns and dependent columns, a la (k1, k2, k3 => d1, d2, d3)
#[macro_export]
macro_rules! ght_tup {
    // Flat tuple base case: when there is exactly one field
    // First the version with a dereference on a
    (&$a:expr) => {
        var_expr!(&var_expr!($a))
    };
    // Then the version without a dereference on b
    ($a:expr) => {
        var_expr!(var_expr!($a))
    };
    // Flat tuple base case2: when there are exactly two fields
    // First the version with a dereference on b
    ($a:expr, &$b:expr) => {
        var_expr!($a, &var_expr!($b))
    };
    // Then the version without a dereference on b
    ($a:expr, $b:expr) => {
        var_expr!($a, var_expr!($b))
    };

    // Flat tuple recursive case: when there are more than two fields
    ($a:expr, $($rest:tt)*) => {
        var_expr!($a, ...ght_tup!($($rest)*))
    };

    // arrow delimiter
    ($( $a:expr ),* => $( $b:expr ),*) => (
        var_expr!($( $a ),*, var_expr!($( $b ),*))
    );

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        // Example usage
        // let htrie1 = ght_tup!(42, 314, 43770).to_hash_trie();
        let htrie1 = ght_tup!(42, 314, "hello").to_hash_trie();
        assert!(htrie1.contains(var_expr!(&42, &314, &"hello")));
        assert_eq!(htrie1.len(), 1);

        // let tuple2 = ght_tup!(42, 30619);
        let tuple2 = ght_tup!(42, 314);
        let htrie2 = tuple2.to_hash_trie();
        assert!(htrie2.contains(var_expr!(&42, &314)));
        assert_eq!(htrie2.len(), 1);

        let tuple3 = ght_tup!("Rust");
        let htrie3 = tuple3.to_hash_trie();
        assert!(htrie3.contains(var_expr!(&"Rust")));
        assert_eq!(htrie3.len(), 1);
    }

    #[test]
    fn test_hash_trie_type_macro() {
        type LilTrie = GeneralizedHashTrie!(u32);
        // type LilVTrie = GeneralizedHashTrieVariadic!(var_type!(u32));
        let _l: LilTrie = ght_tup!(1).to_hash_trie();
        // let _: LilVTrie = l;

        type SmallTrie = GeneralizedHashTrie!(u32, &'static str);
        // type SmallVTrie = GeneralizedHashTrieVariadic!(var_type!(u32, &'static str));
        type SmallKeyedTrie = GeneralizedHashTrie!(u32 => &'static str);
        // type SmallKeyedVTrie =
        //     GeneralizedHashTrieVariadic!(var_type!(u32 => var_type!(&'static str)));
        let l: SmallTrie = ght_tup!(1, "hello").to_hash_trie();
        // let _: SmallVTrie = l;
        let _: SmallKeyedTrie = l;
        // let _: SmallKeyedVTrie = l;

        type LongKeySmallValTrie = GeneralizedHashTrie!(u32, u16, &'static str);
        // type LongKeySmallValVTrie = GeneralizedHashTrieVariadic!(var_type!(u32, u16, &'static str));
        type LongKeySmallValKeyedTrie = GeneralizedHashTrie!(u32, u16 => &'static str);
        // type LongKeySmallValKeyedVTrie =
        //     GeneralizedHashTrieVariadic!(var_type!(u32, u16 => var_type!(&'static str)));
        let x: LongKeySmallValTrie = ght_tup!(1, 314, "hello").to_hash_trie();
        // let _: LongKeySmallValVTrie = x;
        let _: LongKeySmallValKeyedTrie = x;
        // let _: LongKeySmallValKeyedVTrie = x;
        let _: LongKeySmallValTrie = ght_tup!(1, 314 => "hello").to_hash_trie();
        // let _: LongKeySmallValKeyedVTrie = ght_tup!(1, 314 => "hello").to_hash_trie();

        type SmallKeyLongValTrie = GeneralizedHashTrie!(u32 => u64, u16, &'static str);
        // type SmallKeyLongValVTrie =
        //     GeneralizedHashTrieVariadic!(var_type!(u32 => var_type!(u64, u16, &'static str)));
        let _x: SmallKeyLongValTrie = ght_tup!(1 => 999, 222, "hello").to_hash_trie();
        // let _: SmallKeyLongValVTrie = x;

        type LongKeyLongValTrie = GeneralizedHashTrie!(u32, u64 => u16, &'static str);
        // type LongKeyLongValVTrie =
        //     GeneralizedHashTrieVariadic!(var_type!(u32, u64 => var_type!(u16, &'static str)));
        let _x: LongKeyLongValTrie = ght_tup!(1, 999 => 222, "hello").to_hash_trie();
        // let _: LongKeyLongValVTrie = x;
    }

    #[test]
    fn test_ght_tup() {
        // base case: no reference
        let x = ght_tup!(1);
        let y = var_expr!(var_expr!(1));
        assert_eq!(x, y);

        // base case: reference
        let x = ght_tup!(&1);
        let y = var_expr!(&var_expr!(1));
        assert_eq!(x, y);

        // base case 2: no reference, both syntaxes
        let x = ght_tup!(1, "hello", true => -5);
        let y = ght_tup!(1, "hello", true, -5);
        assert_eq!(x, y);

        // base case 2: reference
        let x = ght_tup!(1, &true);
        let y = var_expr!(1, &var_expr!(true));
        assert_eq!(x, y);

        // recursive case: no reference, => syntax
        let x = ght_tup!(1, "hello" => true, -5);
        let y = var_expr!(1, "hello", var_expr!(true, -5));
        assert_eq!(x, y);

        // // recursive case: reference, => syntax
        let x = ght_tup!(1, "hello" => &true, &-5);
        let y = var_expr!(1, "hello", var_expr!(&true, &-5));
        assert_eq!(x, y);
    }

    #[test]
    fn test_insert() {
        let mut htrie = <GeneralizedHashTrie!(u16, u32, u64)>::default();
        htrie.insert(var_expr!(42, 314, 43770));
        assert_eq!(htrie.len(), 1);
        htrie.insert(var_expr!(42, 315, 43770));
        assert_eq!(htrie.len(), 2);
        htrie.insert(var_expr!(42, 314, 30619));
        assert_eq!(htrie.len(), 3);
        htrie.insert(var_expr!(43, 10, 600));
        assert_eq!(htrie.len(), 4);
        assert!(htrie.contains(var_expr!(&42, &314, &30619)));
        assert!(htrie.contains(var_expr!(&42, &315, &43770)));
        assert!(htrie.contains(var_expr!(&43, &10, &600)));

        type LongKeyLongValTrie = GeneralizedHashTrie!(u32, u64 => u16, &'static str);
        let mut htrie: LongKeyLongValTrie = ght_tup!(1, 999 => 222, "hello").to_hash_trie();
        htrie.insert(var_expr!(1, 999, 111, "bye"));
        htrie.insert(var_expr!(1, 1000, 123, "cya"));
        println!("htrie: {:?}", htrie);
        assert!(htrie.contains(var_expr!(&1, &999, &222, &"hello")));
        assert!(htrie.contains(var_expr!(&1, &999, &111, &"bye")));
        assert!(htrie.contains(var_expr!(&1, &1000, &123, &"cya")));
    }

    #[test]
    fn test_scale() {
        let mut htrie = ght_tup!(true, 1, "hello", -5).to_hash_trie();
        assert_eq!(htrie.len(), 1);
        for i in 1..1000000 {
            htrie.insert(var_expr!(true, 1, "hello", i));
        }
        assert_eq!(htrie.len(), 1000000);
        assert_eq!(htrie.recursive_iter().count(), 1000000);
    }

    #[test]
    fn test_contains() {
        let htrie = ght_tup!(42_u16, 314_u32, 43770_u64).to_hash_trie();
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
        let mut htrie = ght_tup!(42, 314, 43770).to_hash_trie();
        htrie.insert(var_expr!(42, 315, 43770));
        htrie.insert(var_expr!(42, 314, 30619));
        htrie.insert(var_expr!(43, 10, 600));
        for row in htrie.recursive_iter() {
            println!("{:?}", row);
        }
    }

    #[test]
    fn test_get() {
        let ht_root = ght_tup!(42, 314, 43770).to_hash_trie();

        let inner = ht_root.get(&42).unwrap();
        let t = inner.recursive_iter().next().unwrap();
        assert_eq!(t, var_expr!(&314, &43770));

        let leaf = inner.get(&314).unwrap();
        let t = leaf.recursive_iter().next().unwrap();
        assert_eq!(t, var_expr!(&43770));
    }

    #[test]
    fn test_iter() {
        let ht_root = ght_tup!(42, 314, 43770).to_hash_trie();
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
        let mut htrie = ght_tup!(42, 314, 43770).to_hash_trie();
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
        // // htrie.prefix_iter(ght_tup!(&42, &315, &43770, &100));

        for row in htrie.recursive_iter() {
            println!("All: {:?}", row);
        }
    }

    #[test]
    fn test_prefix_iter_complex() {
        let mut htrie = ght_tup!(true, 1, "hello", -5).to_hash_trie();
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
    // #[test]
    // fn test_set_union_ght() {
    //     type MyGHT = GeneralizedHashTrie!(u32, u64 => u16, &'static str);

    //     let mut test_ght1: MyGHT = ght_tup!(42, 314 => 10, "hello").to_hash_trie();
    //     let mut test_ght2: MyGHT = ght_tup!(42, 314 => 10, "hello").to_hash_trie();

    //     test_ght1.insert(var_expr!(42, 314, 20, "goodbye"));
    //     test_ght2.insert(var_expr!(42, 314, 20, "again"));

    //     let mug1 = SetUnionGHT::new(test_ght1);
    //     let mug2 = SetUnionGHT::new(test_ght2);

    //     mug1.merge(mug2);
    //     println!("merged: {:?}", mug1);
    // }
}
