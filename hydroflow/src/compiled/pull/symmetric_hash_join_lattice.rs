use crate::lang::lattice::{Convert, LatticeRepr, Merge};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{
    collections::{hash_map::Entry, hash_set},
    marker::PhantomData,
};

#[derive(Debug)]
pub struct HalfJoinStateLattice<Key, LR: LatticeRepr + Merge<LRD>, LRD: LatticeRepr> {
    table: FxHashMap<Key, LR::Repr>,
    _marker: PhantomData<*const LRD>,
}

impl<Key, LR: LatticeRepr + Merge<LRD>, LRD: LatticeRepr> Default
    for HalfJoinStateLattice<Key, LR, LRD>
{
    fn default() -> Self {
        Self {
            table: FxHashMap::default(),
            _marker: PhantomData::default(),
        }
    }
}
impl<Key, LR: LatticeRepr + Merge<LRD> + Convert<LRD>, LRD: LatticeRepr>
    HalfJoinStateLattice<Key, LR, LRD>
where
    Key: Clone + Eq + std::hash::Hash,
    LR::Repr: Clone + Eq,
{
    fn build(&mut self, k: Key, v: LR::Repr) -> bool {
        let entry = self.table.entry(k);

        match entry {
            Entry::Occupied(mut e) => {
                return <LR as Merge<LRD>>::merge(e.get_mut(), <LR as Convert<LRD>>::convert(v));
            }
            Entry::Vacant(e) => {
                e.insert(v);
                return true;
            }
        };
    }
}

pub type _JoinStateLattice<Key, V1: LatticeRepr, V1D, V2: LatticeRepr, V2D> = (
    HalfJoinStateLattice<Key, V1, V1D>,
    HalfJoinStateLattice<Key, V2, V2D>,
);
pub type JoinStateLatticeMut<'a, Key, V1: LatticeRepr, V1D, V2: LatticeRepr, V2D> = (
    &'a mut HalfJoinStateLattice<Key, V1, V1D>,
    &'a mut HalfJoinStateLattice<Key, V2, V2D>,
);

pub struct SymmetricHashJoinLattice<'a, Key, V1, V1D, V2, V2D>
where
    Key: Eq + std::hash::Hash + Clone,
    V1: Merge<V1D>,
    V1::Repr: Eq + Clone,
    V1D: LatticeRepr,
    V1D::Repr: Eq + Clone,
    V2: Merge<V2D>,
    V2::Repr: Eq + Clone,
    V2D: LatticeRepr,
    V2D::Repr: Eq + Clone,
{
    state: JoinStateLatticeMut<'a, Key, V1, V1D, V2, V2D>,
    updated_keys: hash_set::IntoIter<Key>,
}

impl<'a, Key, V1, V1D, V2, V2D> Iterator for SymmetricHashJoinLattice<'a, Key, V1, V1D, V2, V2D>
where
    Key: Eq + std::hash::Hash + Clone,
    V1: Merge<V1D> + Convert<V1D>,
    V1::Repr: Eq + Clone,
    V1D: LatticeRepr,
    V1D::Repr: Eq + Clone,
    V2: Merge<V2D> + Convert<V2D>,
    V2::Repr: Eq + Clone,
    V2D: LatticeRepr,
    V2D::Repr: Eq + Clone,
{
    type Item = (Key, (V1::Repr, V2::Repr));

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
impl<'a, Key, V1, V1D, V2, V2D> SymmetricHashJoinLattice<'a, Key, V1, V1D, V2, V2D>
where
    Key: Eq + std::hash::Hash + Clone,
    V1: Merge<V1D> + Convert<V1D>,
    V1::Repr: Eq + Clone,
    V1D: LatticeRepr,
    V1D::Repr: Eq + Clone,
    V2: Merge<V2D> + Convert<V2D>,
    V2::Repr: Eq + Clone,
    V2D: LatticeRepr,
    V2D::Repr: Eq + Clone,
{
    pub fn new_from_mut<I1, I2>(
        mut lhs: I1,
        mut rhs: I2,
        state_lhs: &'a mut HalfJoinStateLattice<Key, V1, V1D>,
        state_rhs: &'a mut HalfJoinStateLattice<Key, V2, V2D>,
    ) -> Self
    where
        I1: Iterator<Item = (Key, V1::Repr)>,
        I2: Iterator<Item = (Key, V2::Repr)>,
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
    use super::{SymmetricHashJoinLattice, _JoinStateLattice as JoinStateLattice};
    use crate::lang::lattice::ord::MaxRepr;

    type JoinStateMaxLattice =
        JoinStateLattice<usize, MaxRepr<usize>, MaxRepr<usize>, MaxRepr<usize>, MaxRepr<usize>>;

    fn join<X: IntoIterator<Item = (usize, usize)>, Y: IntoIterator<Item = (usize, usize)>>(
        state: &mut JoinStateMaxLattice,
        lhs: X,
        rhs: Y,
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
