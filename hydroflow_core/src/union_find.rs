use slotmap::{Key, SecondaryMap};

#[derive(Default, Clone)]
pub struct UnionFind<K>
where
    K: Key,
{
    links: SecondaryMap<K, K>,
}
impl<K> UnionFind<K>
where
    K: Key,
{
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            links: SecondaryMap::with_capacity(capacity),
        }
    }

    pub fn union(&mut self, a: K, b: K) {
        let i = self.find(a);
        let j = self.find(b);
        if i == j {
            return;
        }
        self.links[i] = j;
    }
    pub fn find(&mut self, k: K) -> K {
        if let Some(next) = self.links.insert(k, k) {
            if k == next {
                return k;
            }
            self.links[k] = self.find(next);
        }
        self.links[k]
    }
    pub fn same_set(&mut self, a: K, b: K) -> bool {
        self.find(a) == self.find(b)
    }
}
impl<K> FromIterator<K> for UnionFind<K>
where
    K: Key,
{
    fn from_iter<T: IntoIterator<Item = K>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let mut uf = iter
            .size_hint()
            .1
            .map(Self::with_capacity)
            .unwrap_or_default();

        for k in iter {
            uf.union(k, k)
        }

        uf
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use slotmap::SlotMap;

    #[test]
    fn test_basic() {
        let mut sm = SlotMap::new();
        let a = sm.insert(());
        let b = sm.insert(());
        let c = sm.insert(());
        let d = sm.insert(());

        let mut uf = UnionFind::new();
        assert!(!uf.same_set(a, b));
        uf.union(a, b);
        assert!(uf.same_set(a, b));
        uf.union(c, a);
        assert!(uf.same_set(b, c));

        assert!(!uf.same_set(a, d));
        assert!(!uf.same_set(b, d));
        assert!(!uf.same_set(d, c));
    }
}
