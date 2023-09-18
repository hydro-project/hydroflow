use std::collections::hash_map::Entry::*;

use rustc_hash::FxHashMap;
use smallvec::{smallvec, SmallVec};

use crate::util::clear::Clear;

pub struct HalfJoinStateMultiset<K, V> {
    pub table: FxHashMap<K, SmallVec<[V; 1]>>,
}

impl<K, V> Default for HalfJoinStateMultiset<K, V> {
    fn default() -> Self {
        Self {
            table: Default::default(),
        }
    }
}

impl<K, V> Clear for HalfJoinStateMultiset<K, V> {
    fn clear(&mut self) {
        self.table.clear()
    }
}

impl<K, V> HalfJoinStateMultiset<K, V>
where
    K: Eq + std::hash::Hash,
{
    pub fn push(&mut self, iter: impl Iterator<Item = (K, V)>) {
        for (k, v) in iter {
            let entry = self.table.entry(k);

            match entry {
                Occupied(mut e) => e.get_mut().push(v),
                Vacant(e) => {
                    e.insert(smallvec![v]);
                }
            }
        }
    }
}
