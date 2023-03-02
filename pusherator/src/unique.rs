use std::{cell::RefMut, collections::HashSet, hash::Hash};

use super::Pusherator;

pub struct Unique<'a, Next>
where
    Next: Pusherator,
    Next::Item: Clone + Eq + PartialEq + Hash,
{
    next: Next,
    state: RefMut<'a, HashSet<Next::Item>>,
}
impl<Next> Pusherator for Unique<'_, Next>
where
    Next: Pusherator,
    Next::Item: Clone + Eq + PartialEq + Hash,
{
    type Item = Next::Item;
    fn give(&mut self, item: Self::Item) {
        if self.state.insert(item.clone()) {
            self.next.give(item);
        }
    }
}
impl<'a, Next> Unique<'a, Next>
where
    Next: Pusherator,
    Next::Item: Clone + Eq + PartialEq + Hash,
{
    pub fn new(state: RefMut<'a, HashSet<Next::Item>>, next: Next) -> Self {
        Self { next, state }
    }
}
