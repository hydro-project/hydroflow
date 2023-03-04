use crdts::CvRDT;

type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

#[derive(Debug)]
pub struct HalfJoinStateHack<Key, ValBuild> {
    table: HashMap<Key, ValBuild>,
}
impl<Key, ValBuild> Default for HalfJoinStateHack<Key, ValBuild> {
    fn default() -> Self {
        Self {
            table: HashMap::default(),
        }
    }
}

pub type JoinStateHack<Key, V1, V2> = (HalfJoinStateHack<Key, V1>, HalfJoinStateHack<Key, V2>);
pub type JoinStateHackMut<'a, Key, V1, V2> = (
    &'a mut HalfJoinStateHack<Key, V1>,
    &'a mut HalfJoinStateHack<Key, V2>,
);

pub struct SymmetricHashJoinHack<'a, Key, I1, V1, I2, V2>
where
    Key: Eq + std::hash::Hash + Clone,
    V1: Eq + Clone,
    V2: Eq + Clone,
    I1: Iterator<Item = (Key, V1)>,
    I2: Iterator<Item = (Key, V2)>,
{
    lhs: I1,
    rhs: I2,
    state: JoinStateHackMut<'a, Key, V1, V2>,
}

impl<'a, Key, I1, V1, I2, V2> Iterator for SymmetricHashJoinHack<'a, Key, I1, V1, I2, V2>
where
    Key: Eq + std::hash::Hash + Clone,
    V1: Eq + Clone + CvRDT + Default,
    V2: Eq + Clone + CvRDT + Default,
    I1: Iterator<Item = (Key, V1)>,
    I2: Iterator<Item = (Key, V2)>,
{
    type Item = (Key, (V1, V2));

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // This only works because it processes the lhs entirely and firstly. So it also only works with 'static, 'tick joins.
            if let Some((k, v1)) = self.lhs.next() {
                self.state
                    .0
                    .table
                    .entry(k.clone())
                    .or_default()
                    .merge(v1.clone());
                if let Some(v2) = self.state.1.table.get(&k) {
                    return Some((k, (v1, v2.clone())));
                }
            } else if let Some((k, v2)) = self.rhs.next() {
                self.state
                    .1
                    .table
                    .entry(k.clone())
                    .or_default()
                    .merge(v2.clone());
                if let Some(v1) = self.state.0.table.get(&k) {
                    return Some((k, (v1.clone(), v2)));
                }
            } else {
                return None;
            }
        }
    }
}
impl<'a, Key, I1, V1, I2, V2> SymmetricHashJoinHack<'a, Key, I1, V1, I2, V2>
where
    Key: Eq + std::hash::Hash + Clone,
    V1: Eq + Clone + CvRDT + Default,
    V2: Eq + Clone + CvRDT + Default,
    I1: Iterator<Item = (Key, V1)>,
    I2: Iterator<Item = (Key, V2)>,
{
    pub fn new(lhs: I1, rhs: I2, state: &'a mut JoinStateHack<Key, V1, V2>) -> Self {
        Self {
            lhs,
            rhs,
            state: (&mut state.0, &mut state.1),
        }
    }

    pub fn new_from_mut(
        lhs: I1,
        rhs: I2,
        state_lhs: &'a mut HalfJoinStateHack<Key, V1>,
        state_rhs: &'a mut HalfJoinStateHack<Key, V2>,
    ) -> Self {
        Self {
            lhs,
            rhs,
            state: (state_lhs, state_rhs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{JoinStateHack, SymmetricHashJoinHack};
    use crdts::GSet;

    #[test]
    fn hash_join() {
        let mut lhs = GSet::new();
        lhs.insert(0);
        lhs.insert(1);

        let mut rhs1 = GSet::new();
        rhs1.insert(1);
        let mut rhs2 = GSet::new();
        rhs2.insert(2);

        let lhs = [(1, lhs)].into_iter();
        let rhs = [(1, rhs1), (1, rhs2)].into_iter();

        let mut state = JoinStateHack::default();
        let join = SymmetricHashJoinHack::new(lhs, rhs, &mut state);

        // assert_eq!(
        //     join.collect::<Vec<_>>(),
        //     vec![(1, (GSet::new(), GSet::new()))]
        // );
    }
}
