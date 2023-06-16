//! A vector that supports efficient deletion without reordering all subsequent items.
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Default)]
/// A vector that supports efficient deletion without reordering all subsequent items.
pub struct SparseVec<T> {
    vec: Vec<Option<T>>,
    hash_map: HashMap<T, Vec<usize>>,
}

impl<T: Clone + Eq + Hash> SparseVec<T> {
    /// Insert item into the vector, see `https://doc.rust-lang.org/std/vec/struct.Vec.html#method.push`
    pub fn push(&mut self, item: T) {
        self.vec.push(Some(item.clone()));
        self.hash_map
            .entry(item)
            .or_insert(Vec::with_capacity(1))
            .push(self.vec.len() - 1);
    }

    /// Delete all items of a specific value from this vector. This takes time proportional to the amount of items of that value in the vector, not the total size of th vector.
    pub fn delete(&mut self, item: &T) {
        if let Some(indices) = self.hash_map.get(item) {
            for index in indices {
                self.vec[*index] = None;
            }
        }
    }

    /// Iterate through all items in the vector in order. Deleted items will not appear in the iteration.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.vec.iter().filter_map(|x| x.as_ref())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn collect<T: Eq + Hash + Clone>(sv: &SparseVec<T>) -> Vec<T> {
        sv.iter().cloned().collect()
    }

    #[test]
    fn basic() {
        let mut x = SparseVec::default();

        x.push(0);
        x.push(1);
        x.push(2);

        x.delete(&1);

        assert_eq!(collect(&x), vec![0, 2]);
    }
}
