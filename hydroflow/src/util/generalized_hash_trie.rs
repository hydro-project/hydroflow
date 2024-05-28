use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use variadics::{var_expr, var_type, AsRefVariadic, Variadic, VariadicExt};

/// node of a HashTrie
pub trait HashTrieNode: Default {
    type Items<'a>: Variadic + AsRefVariadic<'a>;

    /// Inserts items into the hash trie.
    /// Returns `true` if the items were successfully inserted, `false` otherwise.
    fn insert(&mut self, items: &[Self::Items<'static>]) -> bool;

    /// Performs a prefix search on the hash trie.
    /// Returns `true` if the prefix is found, `false` otherwise.
    fn prefix_search(&self, prefix: &[&T]) -> bool;

    /// Item for [`Self::Iter`].
    type Item;
    /// Iterator for [`Self::iter`].
    type Iter<'a>: Iterator<Item = Self::Item>
    where
        Self: 'a;
    /// Iterate the sub-tree.
    fn iter(&self) -> Self::Iter<'_>;
}

/// internal node of a HashTrie
#[derive(Debug)]
pub struct HTInner<H: 'static, T: HashTrieNode<H>> {
    children: HashMap<H, T>,
}
impl<H: 'static, T: HashTrieNode<H>> Default for HTInner<H, T> {
    fn default() -> Self {
        let children = Default::default();
        Self { children }
    }
}

impl<H, T: 'static + HashTrieNode<H>> HashTrieNode<H> for HTInner<H, T>
where
    H: 'static + Eq + Hash + Clone,
{
    fn prefix_search(&self, prefix: &[&H]) -> bool {
        if prefix.is_empty() {
            return true;
        }
        let head = prefix[0];
        if let Some(node) = self.children.get(head) {
            node.prefix_search(&prefix[1..])
        } else {
            false
        }
    }

    fn insert(&mut self, items: &[&H]) -> bool {
        if items.is_empty() {
            return false;
        }

        let head = items[0];
        self.children
            .entry(head.clone())
            .or_default()
            .insert(&items[1..])
    }

    type Item = var_type!(H, ...T::Item);
    type Iter<'a> = HTInnerIter<'a, H, T>;
    fn iter(&self) -> Self::Iter<'_> {
        self.children
            .iter()
            .flat_map(|(k, vs)| vs.iter().map(move |v| var_expr!(k.clone(), ...v)))
    }
}
/// Iter for HTLeaf.
pub type HTInnerIter<'a, H: 'static + Eq + Hash + Clone, T: 'static + HashTrieNode<H>> =
    impl 'a + Iterator<Item = var_type!(H, ...T::Item)>;

/// leaf node of a HashTrie
#[derive(Debug)]
pub struct HTLeaf<T> {
    elements: Vec<T>,
}
impl<T> Default for HTLeaf<T> {
    fn default() -> Self {
        let elements = Default::default();
        Self { elements }
    }
}

impl<T> HashTrieNode<T> for HTLeaf<T>
where
    T: 'static + Clone + PartialEq,
{
    fn prefix_search(&self, prefix: &[&T]) -> bool {
        if prefix.is_empty() {
            return true;
        }
        let head = prefix[0];
        self.elements.contains(head)
    }

    fn insert(&mut self, items: &[&T]) -> bool {
        if !items.is_empty() {
            let val = items[0];
            if !self.elements.contains(val) {
                self.elements.push(val.clone());
                return true;
            }
            return false;
        }
        false
    }

    type Item = var_type!(T);
    type Iter<'a> = HTLeafIter<'a, T>;
    fn iter(&self) -> Self::Iter<'_> {
        self.elements.iter().map(|item| var_expr!(item.clone()))
    }
}
/// Iter for HTLeaf.
pub type HTLeafIter<'a, T: 'static + Clone + PartialEq> = impl 'a + Iterator<Item = var_type!(T)>;

/// Trait to convert a tuple to a HashTrieList
pub trait ToHashTrie {
    /// The output type of the hash trie conversion.
    type Output: HashTrieNode<Self::Item>;
    /// This allows us to avoid using dyn
    type Item;

    /// Converts the object into a hash trie.
    fn to_hash_trie(self) -> Self::Output;
}

impl<Head> ToHashTrie for var_type!(Head)
where
    Head: 'static + Eq + Hash + Clone,
{
    type Output = HTLeaf<Head>;
    type Item = Head;

    fn to_hash_trie(self) -> Self::Output {
        let mut result = HTLeaf {
            elements: Vec::new(),
        };
        result.elements.push(self.0.clone());
        result
    }
}

impl<Head, Tail> ToHashTrie for var_type!(Head, ...Tail)
where
    Head: 'static + Eq + Hash + Clone + Debug,
    Tail: 'static + Variadic + ToHashTrie + VariadicExt,
    <Tail as ToHashTrie>::Output: HashTrieNode<Head>,
{
    type Output = HTInner<Head, Tail::Output>;
    type Item = Head;

    fn to_hash_trie(self) -> Self::Output {
        let var_expr!(head, ...tail) = self;
        let mut result = HTInner::<Head, Tail::Output> {
            children: HashMap::<Head, Tail::Output>::new(),
        };
        result.children.insert(head, tail.to_hash_trie());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_test() {
        // Example usage
        // let htrie1 = var_expr!(42, 314, 43770).to_hash_trie();
        let htrie1 = var_expr!(42, 314, 43770).to_hash_trie();
        println!("HTrie: {:?}", htrie1);

        // let tuple2 = var_expr!(42, 30619);
        let tuple2 = var_expr!(42, 476);
        let htrie2 = tuple2.to_hash_trie();
        println!("HTrie: {:?}", htrie2);

        let tuple3 = var_expr!("Rust");
        let htrie3 = tuple3.to_hash_trie();
        println!("HTrie: {:?}", htrie3);
    }

    // How can I insert non-atomic objects?
    #[test]
    fn test_search_prefix() {
        let htrie = var_expr!(42, 314, 43770).to_hash_trie();
        println!("HTrie: {:?}", htrie);
        assert!(htrie.prefix_search(&[&42]));
        assert!(!htrie.prefix_search(&[&43]));
        assert!(htrie.prefix_search(&[&42, &314]));
        assert!(!htrie.prefix_search(&[&42, &315]));
        assert!(!htrie.prefix_search(&[&43, &314]));
        assert!(htrie.prefix_search(&[&42, &314, &43770]));
        assert!(!htrie.prefix_search(&[&42, &314, &30619]));
        assert!(!htrie.prefix_search(&[&42, &315, &43770]));
        assert!(!htrie.prefix_search(&[&43, &314, &43770]));
    }

    #[test]
    fn test_insert() {
        let mut htrie = var_expr!(42, 314, 43770).to_hash_trie();
        assert!(htrie.insert(&[&42, &315, &43770]));
        assert!(htrie.insert(&[&42, &314, &30619]));
        assert!(htrie.insert(&[&43, &10, &600]));
        assert!(htrie.prefix_search(&[&42, &314, &30619]));
        assert!(!htrie.prefix_search(&[&42, &314, &600]));
        assert!(htrie.prefix_search(&[&42, &315]));
        assert!(htrie.prefix_search(&[&42, &315, &43770]));
        assert!(htrie.prefix_search(&[&43]));
        assert!(htrie.prefix_search(&[&43, &10]));
        assert!(htrie.prefix_search(&[&43, &10, &600]));
    }

    #[test]
    fn test_iter() {
        let mut htrie = var_expr!(42, 314, 43770).to_hash_trie();
        assert!(htrie.insert(&[&42, &315, &43770]));
        assert!(htrie.insert(&[&42, &314, &30619]));
        assert!(htrie.insert(&[&43, &10, &600]));
        for row in htrie.iter() {
            println!("{:?}", row);
        }
    }
}
