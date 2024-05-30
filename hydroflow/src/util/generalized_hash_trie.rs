use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use sealed::sealed;
use variadics::{var_args, var_expr, var_type, Variadic, VariadicExt};

// pub fn flatten_tuple(v: Variadic) -> Variadic {
//     let (head, tail) = v;

// }
/// node of a HashTrie
#[sealed]
pub trait HashTrieNode: Default {
    /// Row variadic.
    type Row: VariadicExt;

    /// Inserts items into the hash trie.
    fn insert(&mut self, row: Self::Row);

    /// Returns `true` if the row is found, `false` otherwise.
    fn contains(&self, row: <Self::Row as VariadicExt>::AsRefVar<'_>) -> bool;

    /// Iterator for [`Self::iter`].
    type Iter<'a>: Iterator<Item = <Self::Row as VariadicExt>::AsRefVar<'a>>
    where
        Self: 'a;
    /// Iterate the sub-tree.
    fn iter(&self) -> Self::Iter<'_>;
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

    type Iter<'a> = HtInnerIter<'a, Key, Node>;
    fn iter(&self) -> Self::Iter<'_> {
        self.children
            .iter()
            .flat_map(|(k, vs)| vs.iter().map(move |v| var_expr!(k, ...v)))
    }
}
/// Iter for HTLeaf.
pub type HtInnerIter<'a, Key, Node>
where
    Key: 'static + Eq + Hash,
    Node: 'static + HashTrieNode,
= impl 'a
    + Iterator<Item = <<HtInner<Key, Node> as HashTrieNode>::Row as VariadicExt>::AsRefVar<'a>>;

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
    T: 'static + Eq,
{
    type Row = var_expr!(T);

    fn insert(&mut self, row: Self::Row) {
        let var_args!(key) = row;
        self.elements.push(key);
    }

    fn contains(&self, row: <Self::Row as VariadicExt>::AsRefVar<'_>) -> bool {
        let var_args!(key) = row;
        self.elements.contains(key)
    }

    type Iter<'a> = HTLeafIter<'a, T>;
    fn iter(&self) -> Self::Iter<'_> {
        self.elements.iter().map(|item| var_expr!(item))
    }
}
/// Iter for HTLeaf.
pub type HTLeafIter<'a, T>
where
    T: 'static + Eq,
= impl 'a + Iterator<Item = <<HtLeaf<T> as HashTrieNode>::Row as VariadicExt>::AsRefVar<'a>>;

#[sealed]
pub trait HtPrefixIter<Prefix> {
    type Suffix: VariadicExt;
    type PrefixIter<'a>: Iterator<Item = <Self::Suffix as VariadicExt>::AsRefVar<'a>>
    where
        Self: 'a,
        Self::Suffix: 'a;
    fn prefix_iter(&self, prefix: Prefix) -> Self::PrefixIter<'_>;
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
    type PrefixIter<'a> = std::iter::Flatten<std::option::IntoIter<Node::PrefixIter<'a>>>
    where
        Self: 'a,
        Self::Suffix: 'a;
    fn prefix_iter(&self, prefix: var_type!(&'k Key, ...PrefixRest)) -> Self::PrefixIter<'_> {
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
    type PrefixIter<'a> = std::option::IntoIter<()>
    where
        Self: 'a,
        Self::Suffix: 'a;
    fn prefix_iter(&self, prefix: var_type!(&'k Key)) -> Self::PrefixIter<'_> {
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
    type PrefixIter<'a> = <Self as HashTrieNode>::Iter<'a>;
    fn prefix_iter(&self, _prefix: var_type!()) -> Self::PrefixIter<'_> {
        self.iter()
    }
}

/// Trait to convert a tuple to a HashTrieList
pub trait ToHashTrie: Variadic {
    /// The output type of the hash trie conversion.
    type Output: HashTrieNode<Row = Self>;

    /// Converts the object into a hash trie.
    fn to_hash_trie(self) -> Self::Output;
}

impl<Head> ToHashTrie for var_type!(Head)
where
    Head: 'static + Eq + Hash + Clone,
{
    type Output = HtLeaf<Head>;

    fn to_hash_trie(self) -> Self::Output {
        let mut result = HtLeaf {
            elements: Vec::new(),
        };
        result.elements.push(self.0.clone());
        result
    }
}

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

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn do_test() {
        // Example usage
        // let htrie1 = var_expr!(42, 314, 43770).to_hash_trie();
        let htrie1 = var_expr!(42, dec!(3.14), "hello").to_hash_trie();
        println!("HTrie: {:?}", htrie1);

        // let tuple2 = var_expr!(42, 30619);
        let tuple2 = var_expr!(42, dec!(3.14));
        let htrie2 = tuple2.to_hash_trie();
        println!("HTrie: {:?}", htrie2);

        let tuple3 = var_expr!("Rust");
        let htrie3 = tuple3.to_hash_trie();
        println!("HTrie: {:?}", htrie3);
    }

    // How can I insert non-atomic objects?
    #[test]
    fn test_search_prefix() {
        let htrie = var_expr!(42_u16, 314_u32, 43770_u64).to_hash_trie();
        println!("HTrie: {:?}", htrie);
        // assert!(htrie.prefix_search(&[&42]));
        // assert!(!htrie.prefix_search(&[&43]));
        // assert!(htrie.prefix_search(&[&42, &314]));
        // assert!(!htrie.prefix_search(&[&42, &315]));
        // assert!(!htrie.prefix_search(&[&43, &314]));
        assert!(htrie.contains(var_expr!(&42, &314, &43770)));
        assert!(htrie.contains(var_expr!(42, 314, 43770).as_ref_var()));
        assert!(!htrie.contains(var_expr!(&42, &314, &30619)));
        assert!(!htrie.contains(var_expr!(42, 314, 30619).as_ref_var()));
        assert!(!htrie.contains(var_expr!(&42, &315, &43770)));
        assert!(!htrie.contains(var_expr!(&43, &314, &43770)));
    }

    #[test]
    fn test_insert() {
        let mut htrie = var_expr!(42, 314, 43770).to_hash_trie();
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
    fn test_iter() {
        let mut htrie = var_expr!(42, 314, 43770).to_hash_trie();
        htrie.insert(var_expr!(42, 315, 43770));
        htrie.insert(var_expr!(42, 314, 30619));
        htrie.insert(var_expr!(43, 10, 600));
        for row in htrie.iter() {
            println!("{:?}", row);
        }
    }

    #[test]
    fn test_prefix_iter() {
        let mut htrie = var_expr!(42, 314, 43770).to_hash_trie();
        htrie.insert(var_expr!(42, 315, 43770));
        htrie.insert(var_expr!(42, 314, 30619));
        htrie.insert(var_expr!(43, 10, 600));

        for row in htrie.prefix_iter(var_expr!(&42)) {
            println!("42: {:?}", row);
        }

        for row in htrie.prefix_iter(var_expr!(&42, &315)) {
            println!("42,315: {:?}", row);
        }

        for row in htrie.prefix_iter(var_expr!(&42, &315, &43770)) {
            println!("42,315,43770: {:?}", row);
        }
        assert!(htrie
            .prefix_iter(var_expr!(&42, &315, &43770))
            .any(|_| true));
        // // Too long:
        // htrie.prefix_iter(var_expr!(&42, &315, &43770, &100));

        for row in htrie.iter() {
            println!("All: {:?}", row);
        }
    }

    #[test]
    fn test_prefix_iter_complex() {
        let mut htrie = var_expr!(true, 1, "hello", -5).to_hash_trie();
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
    fn test_scale() {
        let mut htrie = var_expr!(true, 1, "hello", -5).to_hash_trie();
        for i in 1..1000000 {
            htrie.insert(var_expr!(true, 1, "hello", i))
        }
        assert!(htrie.iter().count() == 1000000);
    }
}
