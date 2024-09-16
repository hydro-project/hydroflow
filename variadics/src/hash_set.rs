use std::hash::{BuildHasher, Hash, Hasher, RandomState};

use hashbrown::hash_table::{Entry, HashTable};

use crate::{PartialEqVariadic, VariadicExt};

pub struct VariadicHashSet<T, S = RandomState> {
    table: HashTable<T>,
    hasher: S,
}
impl<T> VariadicHashSet<T> {
    pub fn new() -> Self {
        Self {
            table: HashTable::new(),
            hasher: RandomState::default(),
        }
    }
}
impl<T, S> VariadicHashSet<T, S>
where
    T: VariadicExt + PartialEqVariadic,
    for<'a> T::AsRefVar<'a>: Hash,
    S: BuildHasher,
{
    fn get_hash(hasher: &S, ref_var: T::AsRefVar<'_>) -> u64 {
        let mut hasher = hasher.build_hasher();
        ref_var.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get<'a>(&'a self, ref_var: T::AsRefVar<'_>) -> Option<&'a T> {
        let hash = Self::get_hash(&self.hasher, ref_var);
        self.table.find(hash, |item| {
            <T as PartialEqVariadic>::eq_ref(ref_var, item.as_ref_var())
        })
    }

    pub fn insert(&mut self, element: T) -> bool {
        let hash = Self::get_hash(&self.hasher, element.as_ref_var());
        let entry = self.table.entry(
            hash,
            |item| <T as PartialEqVariadic>::eq(&element, &item),
            |item| Self::get_hash(&self.hasher, item.as_ref_var()),
        );
        match entry {
            Entry::Occupied(_occupied_entry) => false,
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(element);
                true
            }
        }
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = T::AsRefVar<'a>> {
        self.table.iter().map(|item| item.as_ref_var())
    }
}
impl<T, S> VariadicHashSet<T, S> {
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            table: HashTable::new(),
            hasher,
        }
    }
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self {
            table: HashTable::with_capacity(capacity),
            hasher,
        }
    }
}
