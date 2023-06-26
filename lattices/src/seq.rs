use std::cmp::Ordering::{self, *};

use cc_traits::Iter;

use crate::{IsBot, LatticeFrom, LatticeOrd, Merge};

/// Sequence compound lattice.
///
/// Contains any number of `Lat` sub-lattices. Sub-lattices are indexed starting at zero, merging
/// combines corresponding sub-lattices and keeps any excess.
///
/// Similar to [`MapUnion<<usize, Lat>>`](super::map_union::MapUnion) but requires the key indices
/// start at `0` and have no gaps.
#[derive(Clone, Debug, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Seq<Lat> {
    seq: Vec<Lat>,
}

impl<Lat> Seq<Lat> {
    /// Create a new `Seq` from a `Vec` of `Lat` instances.
    pub fn new(seq: Vec<Lat>) -> Self {
        Self { seq }
    }

    /// Create a new `Seq` from an `Into<Vec<Lat>>`.
    pub fn new_from(seq: impl Into<Vec<Lat>>) -> Self {
        Self::new(seq.into())
    }

    /// Reveal the inner value as a shared reference.
    pub fn as_reveal_ref(&self) -> &Vec<Lat> {
        &self.seq
    }

    /// Reveal the inner value as an exclusive reference.
    pub fn as_reveal_mut(&mut self) -> &mut Vec<Lat> {
        &mut self.seq
    }

    /// Gets the inner by value, consuming self.
    pub fn into_reveal(self) -> Vec<Lat> {
        self.seq
    }
}

impl<Lat> Default for Seq<Lat> {
    fn default() -> Self {
        Self {
            seq: Default::default(),
        }
    }
}

impl<LatSelf, LatOther> Merge<Seq<LatOther>> for Seq<LatSelf>
where
    LatSelf: Merge<LatOther> + LatticeFrom<LatOther>,
{
    fn merge(&mut self, mut other: Seq<LatOther>) -> bool {
        let mut changed = false;
        // Extend `self` if `other` is longer.
        if self.seq.len() < other.seq.len() {
            self.seq
                .extend(other.seq.drain(self.seq.len()..).map(LatSelf::lattice_from));
            changed = true;
        }
        // Merge intersecting indices.
        for (self_val, other_val) in self.seq.iter_mut().zip(other.seq.into_iter()) {
            changed |= self_val.merge(other_val);
        }
        changed
    }
}

impl<LatSelf, LatOther> LatticeFrom<Seq<LatOther>> for Seq<LatSelf>
where
    LatSelf: LatticeFrom<LatOther>,
{
    fn lattice_from(other: Seq<LatOther>) -> Self {
        Self::new(other.seq.into_iter().map(LatSelf::lattice_from).collect())
    }
}

impl<LatSelf, LatOther> PartialEq<Seq<LatOther>> for Seq<LatSelf>
where
    LatSelf: PartialEq<LatOther>,
{
    fn eq(&self, other: &Seq<LatOther>) -> bool {
        if self.seq.len() != other.seq.len() {
            return false;
        }
        return self
            .seq
            .iter()
            .zip(other.seq.iter())
            .all(|(val_self, val_other)| val_self == val_other);
    }
}

impl<LatSelf, LatOther> PartialOrd<Seq<LatOther>> for Seq<LatSelf>
where
    LatSelf: PartialOrd<LatOther>,
{
    fn partial_cmp(&self, other: &Seq<LatOther>) -> Option<Ordering> {
        let (self_len, other_len) = (self.seq.len(), other.seq.len());
        let mut self_any_greater = other_len < self_len;
        let mut other_any_greater = self_len < other_len;
        for (self_val, other_val) in self.seq.iter().zip(other.seq.iter()) {
            match self_val.partial_cmp(other_val) {
                None => {
                    return None;
                }
                Some(Less) => {
                    other_any_greater = true;
                }
                Some(Greater) => {
                    self_any_greater = true;
                }
                Some(Equal) => {}
            }
            if self_any_greater && other_any_greater {
                return None;
            }
        }
        match (self_any_greater, other_any_greater) {
            (true, false) => Some(Greater),
            (false, true) => Some(Less),
            (false, false) => Some(Equal),
            // We check this one after each loop iteration above.
            (true, true) => unreachable!(),
        }
    }
}
impl<LatSelf, LatOther> LatticeOrd<Seq<LatOther>> for Seq<LatSelf> where
    Self: PartialOrd<Seq<LatOther>>
{
}

impl<Lat> IsBot for Seq<Lat> {
    fn is_bot(&self) -> bool {
        self.seq.is_empty()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
    use crate::set_union::SetUnionHashSet;
    use crate::test::{cartesian_power, check_all};
    use crate::Max;

    #[test]
    fn basic() {
        let mut my_seq_a = Seq::<Max<usize>>::default();
        let my_seq_b = Seq::new(vec![Max::new(9), Max::new(4), Max::new(5)]);
        let my_seq_c = Seq::new(vec![Max::new(2), Max::new(5)]);

        assert!(my_seq_a.merge(my_seq_b.clone()));
        assert!(!my_seq_a.merge(my_seq_b));
        assert!(my_seq_a.merge(my_seq_c.clone()));
        assert!(!my_seq_a.merge(my_seq_c));
    }

    #[test]
    fn consistency() {
        let mut test_vec = vec![Seq::new(vec![] as Vec<SetUnionHashSet<_>>)];

        let vals = [vec![], vec![0], vec![1], vec![0, 1]]
            .map(HashSet::from_iter)
            .map(SetUnionHashSet::new);

        test_vec.extend(
            cartesian_power::<_, 1>(&vals).map(|row| Seq::new(row.into_iter().cloned().collect())),
        );
        test_vec.extend(
            cartesian_power::<_, 2>(&vals).map(|row| Seq::new(row.into_iter().cloned().collect())),
        );

        check_all(&test_vec);
    }
}
