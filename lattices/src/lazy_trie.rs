use std::hash::Hash;

use variadics::{var_expr, var_type, EitherRefVariadic, VariadicExt};

use crate::ght::{GeneralizedHashTrieNode, GhtInner, GhtLeaf};

/// COLT from Wang/Willsey/Suciu
pub trait ColumnLazyTrieNode: GeneralizedHashTrieNode {
    /// into_iter for leaf elements, needed by force below
    fn into_iter(self) -> impl Iterator<Item = Self::Schema>;

    /// pull all the data out of this trie node but retain the reference
    fn drain(&mut self) -> impl Iterator<Item = Self::Schema>;

    /// result of `force`ing a node
    type Force: GeneralizedHashTrieNode;
    /// Force the generation of a parent node, as in the Wang/Willsey/Suciu COLT structure
    fn force(self) -> Self::Force;

    /// Force the generation of a parent node but retain ref to this node
    fn force_drain(&mut self) -> Self::Force;
}

impl<Head, Rest> ColumnLazyTrieNode for GhtLeaf<var_type!(Head, ...Rest)>
where
    Head: 'static + Hash + Eq,
    Rest: 'static + Hash + Eq + VariadicExt,
    for<'r, 's> <var_type!(Head, ...Rest) as VariadicExt>::AsRefVar<'r>:
        PartialEq<<var_type!(Head, ...Rest) as VariadicExt>::AsRefVar<'s>>,
    for<'r, 's> Rest::AsRefVar<'r>: PartialEq<Rest::AsRefVar<'s>>,
{
    fn into_iter(self) -> impl Iterator<Item = Self::Schema> {
        self.elements.into_iter()
    }

    fn drain(&mut self) -> impl Iterator<Item = Self::Schema> {
        self.elements.drain()
    }

    type Force = GhtInner<Head, GhtLeaf<Rest>>;
    fn force(self) -> GhtInner<Head, GhtLeaf<Rest>> {
        let mut retval = Self::Force::default();
        for t in self.into_iter() {
            let var_expr!(h, ...r) = t;
            retval.insert((h, r));
        }
        retval
    }

    fn force_drain(&mut self) -> GhtInner<Head, GhtLeaf<Rest>> {
        let mut retval = Self::Force::default();
        for t in self.drain() {
            let var_expr!(h, ...r) = t;
            retval.insert((h, r));
        }
        retval
    }
}
