use std::hash::Hash;

use variadics::{var_expr, var_type, VariadicExt};

use crate::ght::{GeneralizedHashTrieNode, GhtHasChildren, GhtInner, GhtLeaf};

/// COLT from Wang/Willsey/Suciu
pub trait ColumnLazyTrieNode: GeneralizedHashTrieNode {
    /// into_iter for leaf elements, needed by force below
    fn into_iter(self) -> Option<impl Iterator<Item = Self::Schema>>;

    /// pull all the data out of this trie node but retain the reference
    fn drain(&mut self) -> Option<impl Iterator<Item = Self::Schema>>;

    /// result of `force`ing a node
    type Force: GeneralizedHashTrieNode + GhtHasChildren;
    // + for<'a> HtPrefixIter<<Self::Schema as VariadicExt>::AsRefVar<'a>>;
    /// Force the generation of a parent node, as in the Wang/Willsey/Suciu COLT structure
    fn force(self) -> Option<Self::Force>;

    /// Force the generation of a parent node but retain ref to this node
    fn force_drain(&mut self) -> Option<Self::Force>;
}

impl<Head, Node> ColumnLazyTrieNode for GhtInner<Head, Node>
where
    Head: 'static + Hash + Eq,
    Node: 'static + GeneralizedHashTrieNode,
{
    fn into_iter(self) -> Option<impl Iterator<Item = var_type!(Head, ...Node::Schema)>> {
        None::<Box<dyn Iterator<Item = Self::Schema>>>
    }

    fn drain(&mut self) -> Option<impl Iterator<Item = Self::Schema>> {
        None::<Box<dyn Iterator<Item = Self::Schema>>>
    }

    type Force = GhtInner<Head, Node>;
    fn force(self) -> Option<Self::Force> {
        None
    }

    fn force_drain(&mut self) -> Option<GhtInner<Head, Node>> {
        None
    }
}

impl<Head, Rest> ColumnLazyTrieNode for GhtLeaf<var_type!(Head, ...Rest)>
where
    Head: 'static + Hash + Eq,
    Rest: 'static + Hash + Eq + VariadicExt,
    // for<'r, 's> <var_type!(Head, ...Rest) as VariadicExt>::AsRefVar<'r>:
    //     PartialEq<<var_type!(Head, ...Rest) as VariadicExt>::AsRefVar<'s>>,
    for<'r> Rest::AsRefVar<'r>: PartialEq<Rest::AsRefVar<'r>>,
{
    fn into_iter(self) -> Option<impl Iterator<Item = Self::Schema>> {
        Some(self.elements.into_iter())
    }

    fn drain(&mut self) -> Option<impl Iterator<Item = Self::Schema>> {
        Some(self.elements.drain())
    }

    type Force = GhtInner<Head, GhtLeaf<Rest>>;
    fn force(self) -> Option<Self::Force> {
        let mut retval = Self::Force::default();
        for t in self.into_iter().unwrap() {
            let var_expr!(h, ...r) = t;
            retval.insert((h, r));
        }
        Some(retval)
    }

    fn force_drain(&mut self) -> Option<GhtInner<Head, GhtLeaf<Rest>>> {
        let mut retval = Self::Force::default();
        for t in self.drain().unwrap() {
            let var_expr!(h, ...r) = t;
            retval.insert((h, r));
        }
        Some(retval)
    }
}
