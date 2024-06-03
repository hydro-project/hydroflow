use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use sealed::sealed;
use variadics::{var_args, var_expr, var_type, AsRefVariadicPartialEq, Variadic, VariadicExt};

// pub fn flatten_tuple(v: Variadic) -> Variadic {
//     let (head, tail) = v;

// }
/// node of a HashTrie
#[sealed]
pub trait HashTrieNode: Default {
    /// Row variadic.
    type Row: VariadicExt + AsRefVariadicPartialEq;
    /// The first item in Row.
    type Key;

    /// Inserts items into the hash trie.
    fn insert(&mut self, row: Self::Row);

    /// Returns `true` if the row is found, `false` otherwise.
    fn contains(&self, row: <Self::Row as VariadicExt>::AsRefVar<'_>) -> bool;

    /// Iterator for the keys (inner nodes) or elements (leaf nodes).
    fn iter(&self) -> impl Iterator<Item = &'_ Self::Key>;

    /// Type returned by [`Self::get`].
    type Get: HashTrieNode;
    /// Retrieves the value associated with the given key.
    /// returns an `Option` containing a reference to the value if found, or `None` if not found.
    fn get(&self, key: &Self::Key) -> Option<&'_ Self::Get>;

    /// Iterate the sub-tree.
    fn recursive_iter(&self) -> impl Iterator<Item = <Self::Row as VariadicExt>::AsRefVar<'_>>;
}

/// internal node of a HashTrie
#[derive(Debug)]
pub struct HtInner<Key, Node>
where
    Node: HashTrieNode,
{
    children: HashMap<Key, Node>,
}
impl<Key, Node: HashTrieNode> Default for HtInner<Key, Node>
where
    Node: HashTrieNode,
{
    fn default() -> Self {
        let children = Default::default();
        Self { children }
    }
}
#[sealed]
impl<Key, Node> HashTrieNode for HtInner<Key, Node>
where
    Key: 'static + Hash + Eq,
    Node: 'static + HashTrieNode,
{
    type Row = var_type!(Key, ...Node::Row);
    type Key = Key;

    fn insert(&mut self, row: Self::Row) {
        let var_args!(key, ...rest) = row;

        self.children.entry(key).or_default().insert(rest);
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
#[derive(Debug)]
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
impl<T> HashTrieNode for HtLeaf<T>
where
    T: 'static + Eq + VariadicExt + AsRefVariadicPartialEq,
{
    type Row = T;
    type Key = T;

    fn insert(&mut self, row: Self::Row) {
        self.elements.push(row);
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
    Node: HashTrieNode + HtPrefixIter<PrefixRest>,
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
    This: 'static + HashTrieNode,
{
    type Suffix = <Self as HashTrieNode>::Row;
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
    type Output: HashTrieNode;

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
        };
        result.children.insert(head, tail.to_hash_trie());
        result
    }
}

// // var_expr!(a, b, var_expr!(c))
// // (a, (b, ((c, ()), ())))
// // ^^^^^^^^^^^^^
// // ^^
// //
// trait FlattenLast: Variadic {
//     type FlattenLast: Variadic;
//     fn flatten_last(self) -> Self::FlattenLast;
// }
// impl<Head, Tail> FlattenLast for var_type!(Head, ...Tail)
// where
//     Tail: FlattenLast,
// {
//     type FlattenLast = var_type!(Head, ...Tail::FlattenLast);
//     fn flatten_last(self) -> Self::FlattenLast {
//         let var_args!(head, ...tail) = self;
//         var_expr!(head, ...tail.flatten_last())
//     }
// }
// impl<Nested> FlattenLast for var_type!(Nested)
// where
//     Nested: Variadic,
// {
//     type FlattenLast = Nested;
//     fn flatten_last(self) -> Self::FlattenLast {
//         let var_args!(nested) = self;
//         nested
//     }
// }

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

        // let tuple2 = ght_tup!(42, 30619);
        let tuple2 = ght_tup!(42, 314);
        let htrie2 = tuple2.to_hash_trie();
        assert!(htrie2.contains(var_expr!(&42, &314)));

        let tuple3 = ght_tup!("Rust");
        let htrie3 = tuple3.to_hash_trie();
        assert!(htrie3.contains(var_expr!(&"Rust")));
    }

    #[test]
    fn test_hash_trie_type_macro() {
        type MyTrie = GeneralizedHashTrie!(u32, u16, &'static str);
        type MyKeyedTrie = GeneralizedHashTrie!(u32, u16 => &'static str);
        let x: MyTrie = ght_tup!(1, 314, "hello").to_hash_trie();
        let _: MyKeyedTrie = x;
        let _: MyTrie = ght_tup!(1, 314 => "hello").to_hash_trie();

        type MyLongerTrie = GeneralizedHashTrie!(u32, u64, u16, &'static str);
        type MyLongerTrieWithKey = GeneralizedHashTrie!(u32, u64, u16 => &'static str);
        let x: MyLongerTrie = ght_tup!(1, 999, 222, "hello").to_hash_trie();
        let _: MyLongerTrieWithKey = x;
        let _: MyLongerTrie = ght_tup!(1, 999, 222 => "hello").to_hash_trie();

        type MyLongerKeyedTrie = GeneralizedHashTrie!(u32, u64 => u16, &'static str);
        let _: MyLongerKeyedTrie = ght_tup!(1, 999 => 222, "hello").to_hash_trie();
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
        htrie.insert(var_expr!(42, 315, 43770));
        htrie.insert(var_expr!(42, 314, 30619));
        htrie.insert(var_expr!(43, 10, 600));
        assert!(htrie.contains(var_expr!(&42, &314, &30619)));
        // assert!(!htrie.prefix_search(&[&42, &314, &600]));
        // assert!(htrie.prefix_search(&[&42, &315]));
        assert!(htrie.contains(var_expr!(&42, &315, &43770)));
        // assert!(htrie.prefix_search(&[&43]));
        // assert!(htrie.prefix_search(&[&43, &10]));
        assert!(htrie.contains(var_expr!(&43, &10, &600)));
    }

    #[test]
    fn test_scale() {
        let mut htrie = ght_tup!(true, 1, "hello", -5).to_hash_trie();
        for i in 1..1000000 {
            htrie.insert(var_expr!(true, 1, "hello", i))
        }
        assert!(htrie.recursive_iter().count() == 1000000);
    }

    #[test]
    fn test_search_prefix() {
        let htrie = ght_tup!(42_u16, 314_u32, 43770_u64).to_hash_trie();
        println!("HTrie: {:?}", htrie);
        // assert!(htrie.prefix_search(&[&42]));
        // assert!(!htrie.prefix_search(&[&43]));
        // assert!(htrie.prefix_search(&[&42, &314]));
        // assert!(!htrie.prefix_search(&[&42, &315]));
        // assert!(!htrie.prefix_search(&[&43, &314]));
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
}
