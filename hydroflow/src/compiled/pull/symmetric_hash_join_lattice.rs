use crate::lang::lattice::{Convert, LatticeRepr, Merge};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{
    collections::{hash_map::Entry, hash_set},
    marker::PhantomData,
};

#[derive(Debug)]
pub struct HalfJoinStateLattice<Key, Lr: LatticeRepr + Merge<LrDelta>, LrDelta: LatticeRepr> {
    table: FxHashMap<Key, Lr::Repr>,
    _marker: PhantomData<*const LrDelta>,
}

impl<Key, Lr: LatticeRepr + Merge<LrDelta>, LrDelta: LatticeRepr> Default
    for HalfJoinStateLattice<Key, Lr, LrDelta>
{
    fn default() -> Self {
        Self {
            table: FxHashMap::default(),
            _marker: PhantomData::default(),
        }
    }
}
impl<Key, Lr: LatticeRepr + Merge<LrDelta> + Convert<LrDelta>, LrDelta: LatticeRepr>
    HalfJoinStateLattice<Key, Lr, LrDelta>
where
    Key: Clone + Eq + std::hash::Hash,
    Lr::Repr: Clone + Eq,
{
    fn build(&mut self, k: Key, v: Lr::Repr) -> bool {
        let entry = self.table.entry(k);

        match entry {
            Entry::Occupied(mut e) => {
                <Lr as Merge<LrDelta>>::merge(e.get_mut(), <Lr as Convert<LrDelta>>::convert(v))
            }
            Entry::Vacant(e) => {
                e.insert(v);
                true
            }
        }
    }
}

pub type JoinStateLatticeMut<'a, Key, Lhs: LatticeRepr, LhsDelta, Rhs: LatticeRepr, RhsDelta> = (
    &'a mut HalfJoinStateLattice<Key, Lhs, LhsDelta>,
    &'a mut HalfJoinStateLattice<Key, Rhs, RhsDelta>,
);

pub struct SymmetricHashJoinLattice<'a, Key, Lhs, LhsDelta, Rhs, RhsDelta>
where
    Key: Eq + std::hash::Hash + Clone,
    Lhs: Merge<LhsDelta>,
    Lhs::Repr: Eq + Clone,
    LhsDelta: LatticeRepr,
    LhsDelta::Repr: Eq + Clone,
    Rhs: Merge<RhsDelta>,
    Rhs::Repr: Eq + Clone,
    RhsDelta: LatticeRepr,
    RhsDelta::Repr: Eq + Clone,
{
    state: JoinStateLatticeMut<'a, Key, Lhs, LhsDelta, Rhs, RhsDelta>,
    updated_keys: hash_set::IntoIter<Key>,
}

impl<'a, Key, Lhs, LhsDelta, Rhs, RhsDelta> Iterator
    for SymmetricHashJoinLattice<'a, Key, Lhs, LhsDelta, Rhs, RhsDelta>
where
    Key: Eq + std::hash::Hash + Clone,
    Lhs: Merge<LhsDelta> + Convert<LhsDelta>,
    Lhs::Repr: Eq + Clone,
    LhsDelta: LatticeRepr,
    LhsDelta::Repr: Eq + Clone,
    Rhs: Merge<RhsDelta> + Convert<RhsDelta>,
    Rhs::Repr: Eq + Clone,
    RhsDelta: LatticeRepr,
    RhsDelta::Repr: Eq + Clone,
{
    type Item = (Key, (Lhs::Repr, Rhs::Repr));

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(key) = self.updated_keys.next() {
            if let Some(lhs) = self.state.0.table.get(&key) {
                if let Some(rhs) = self.state.1.table.get(&key) {
                    return Some((key, (lhs.clone(), rhs.clone())));
                }
            }
        }

        None
    }
}
impl<'a, Key, Lhs, LhsDelta, Rhs, RhsDelta>
    SymmetricHashJoinLattice<'a, Key, Lhs, LhsDelta, Rhs, RhsDelta>
where
    Key: Eq + std::hash::Hash + Clone,
    Lhs: Merge<LhsDelta> + Convert<LhsDelta>,
    Lhs::Repr: Eq + Clone,
    LhsDelta: LatticeRepr,
    LhsDelta::Repr: Eq + Clone,
    Rhs: Merge<RhsDelta> + Convert<RhsDelta>,
    Rhs::Repr: Eq + Clone,
    RhsDelta: LatticeRepr,
    RhsDelta::Repr: Eq + Clone,
{
    pub fn new_from_mut<I1, I2>(
        mut lhs: I1,
        mut rhs: I2,
        state_lhs: &'a mut HalfJoinStateLattice<Key, Lhs, LhsDelta>,
        state_rhs: &'a mut HalfJoinStateLattice<Key, Rhs, RhsDelta>,
    ) -> Self
    where
        I1: Iterator<Item = (Key, Lhs::Repr)>,
        I2: Iterator<Item = (Key, Rhs::Repr)>,
    {
        let mut keys = FxHashSet::default();

        loop {
            if let Some((k, v1)) = lhs.next() {
                if state_lhs.build(k.clone(), v1) {
                    keys.insert(k);
                }
                continue;
            }

            if let Some((k, v2)) = rhs.next() {
                if state_rhs.build(k.clone(), v2) {
                    keys.insert(k);
                }
                continue;
            }

            break;
        }

        Self {
            state: (state_lhs, state_rhs),
            updated_keys: keys.into_iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    pub type JoinStateLattice<Key, Lhs: LatticeRepr, LhsDelta, Rhs: LatticeRepr, RhsDelta> = (
        HalfJoinStateLattice<Key, Lhs, LhsDelta>,
        HalfJoinStateLattice<Key, Rhs, RhsDelta>,
    );

    use super::{HalfJoinStateLattice, SymmetricHashJoinLattice};
    use crate::lang::lattice::{ord::MaxRepr, LatticeRepr};

    type JoinStateMaxLattice =
        JoinStateLattice<usize, MaxRepr<usize>, MaxRepr<usize>, MaxRepr<usize>, MaxRepr<usize>>;

    fn join<Lhs: IntoIterator<Item = (usize, usize)>, Rhs: IntoIterator<Item = (usize, usize)>>(
        state: &mut JoinStateMaxLattice,
        lhs: Lhs,
        rhs: Rhs,
    ) -> Vec<(usize, (usize, usize))> {
        SymmetricHashJoinLattice::new_from_mut(
            lhs.into_iter(),
            rhs.into_iter(),
            &mut state.0,
            &mut state.1,
        )
        .collect::<Vec<_>>()
    }

    #[test]
    fn produces_fully_merged_output() {
        let mut state = JoinStateMaxLattice::default();

        let lhs = [(7, 3), (7, 4)];
        let rhs = [(7, 5), (7, 6)];
        assert_eq!(join(&mut state, lhs, rhs), vec![(7, (4, 6))]);
    }

    #[test]
    fn lattice_only_moves_forward() {
        let mut state = JoinStateMaxLattice::default();

        let lhs = [(7, 4), (7, 3)];
        let rhs = [(7, 6), (7, 5)];
        assert_eq!(join(&mut state, lhs, rhs), vec![(7, (4, 6))]);
    }

    #[test]
    fn subsequent_ticks_dont_produce_if_nothing_has_changed() {
        let mut state = JoinStateMaxLattice::default();

        let lhs = [(7, 3)];
        let rhs = [(7, 3)];
        assert_eq!(join(&mut state, lhs, rhs), vec![(7, (3, 3))]);

        let lhs = [(7, 3)];
        let rhs = [(7, 3)];
        assert_eq!(join(&mut state, lhs, rhs), vec![]);
    }

    #[test]
    fn subsequent_ticks_do_produce_if_something_has_changed() {
        let mut state = JoinStateMaxLattice::default();

        let lhs = [(7, 3)];
        let rhs = [(7, 3)];
        assert_eq!(join(&mut state, lhs, rhs), vec![(7, (3, 3))]);

        let lhs = [(7, 3)];
        let rhs = [(7, 4)];
        assert_eq!(join(&mut state, lhs, rhs), vec![(7, (3, 4))]);
    }

    #[test]
    fn resetting_one_side_works() {
        let mut state = JoinStateMaxLattice::default();

        let lhs = [(7, 3)];
        let rhs = [(7, 3)];
        assert_eq!(join(&mut state, lhs, rhs), vec![(7, (3, 3))]);

        std::mem::take(&mut state.1);

        let lhs = [(7, 3)];
        let rhs = [(7, 3)];
        assert_eq!(join(&mut state, lhs, rhs), vec![(7, (3, 3))]);
    }
}
