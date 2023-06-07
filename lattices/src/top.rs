use std::cmp::Ordering;
use std::cmp::Ordering::*;

use super::{ConvertFrom, Merge};
use crate::LatticeOrd;

/// Wraps a lattice in [`Option`], treating [`None`] as a new top element which compares as greater
/// than to all other values.
///
/// This can be used for giving a sensible top element to lattices that don't
/// necessarily have one. Can be used to implement 'tombstones'
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Top<Inner>(pub Option<Inner>);
impl<Inner> Top<Inner> {
    /// Create a new `Bottom` lattice instance from a value.
    pub fn new(val: Inner) -> Self {
        Self(Some(val))
    }

    /// Create a new `Bottom` lattice instance from a value using `Into`.
    pub fn new_from(val: impl Into<Inner>) -> Self {
        Self::new(val.into())
    }
}

// Cannot auto derive because the generated implementation has the wrong trait bounds.
// https://github.com/rust-lang/rust/issues/26925
impl<Inner> Default for Top<Inner> {
    fn default() -> Self {
        Self(None)
    }
}

impl<Inner, Other> Merge<Top<Other>> for Top<Inner>
where
    Inner: Merge<Other> + ConvertFrom<Other>,
{
    fn merge(&mut self, other: Top<Other>) -> bool {
        match (&mut self.0, other.0) {
            (None, None) => false,
            (this @ Some(_), None) => {
                *this = None;
                true
            }
            (None, Some(_)) => false,
            (Some(self_inner), Some(other_inner)) => self_inner.merge(other_inner),
        }
    }
}

impl<Inner, Other> ConvertFrom<Top<Other>> for Top<Inner>
where
    Inner: ConvertFrom<Other>,
{
    fn from(other: Top<Other>) -> Self {
        Self(other.0.map(Inner::from))
    }
}

impl<Inner, Other> PartialOrd<Top<Other>> for Top<Inner>
where
    Inner: PartialOrd<Other>,
{
    fn partial_cmp(&self, other: &Top<Other>) -> Option<Ordering> {
        match (&self.0, &other.0) {
            (None, None) => Some(Equal),
            (None, Some(_)) => Some(Greater),
            (Some(_), None) => Some(Less),
            (Some(this_inner), Some(other_inner)) => this_inner.partial_cmp(other_inner),
        }
    }
}
impl<Inner, Other> LatticeOrd<Top<Other>> for Top<Inner> where Self: PartialOrd<Top<Other>> {}

impl<Inner, Other> PartialEq<Top<Other>> for Top<Inner>
where
    Inner: PartialEq<Other>,
{
    fn eq(&self, other: &Top<Other>) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => false,
            (Some(this_inner), Some(other_inner)) => this_inner == other_inner,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::set_union::{SetUnionHashSet, SetUnionSingletonSet};
    use crate::test::check_all;

    #[test]
    fn test_singly_nested_singleton_example() {
        let mut my_hash_set = Top::new(SetUnionHashSet::<&str>::default());
        let my_delta_set = Top::new(SetUnionSingletonSet::new_from("hello world"));

        assert!(my_hash_set.merge(my_delta_set)); // Changes
        assert!(!my_hash_set.merge(my_delta_set)); // No changes
    }

    #[test]
    fn test_doubly_nested_singleton_example() {
        let mut my_hash_set = Top::new(Top::new(SetUnionHashSet::<&str>::default()));
        let my_delta_set = Top::new(Top::new(SetUnionSingletonSet::new_from("hello world")));

        assert!(my_hash_set.merge(my_delta_set)); // Changes
        assert!(!my_hash_set.merge(my_delta_set)); // No changes
    }

    #[test]
    #[rustfmt::skip]
    fn auto_derives() {
        type B = Top<SetUnionHashSet<usize>>;

        assert_eq!(B::default().partial_cmp(&B::default()), Some(Equal));
        assert_eq!(B::new(SetUnionHashSet::new_from([])).partial_cmp(&B::default()), Some(Less));
        assert_eq!(B::default().partial_cmp(&B::new_from(SetUnionHashSet::new_from([]))), Some(Greater));
        assert_eq!(B::new(SetUnionHashSet::new_from([])).partial_cmp(&B::new(SetUnionHashSet::new_from([]))), Some(Equal));
        assert_eq!(B::new(SetUnionHashSet::new_from([0])).partial_cmp(&B::new(SetUnionHashSet::new_from([]))), Some(Greater));
        assert_eq!(B::new(SetUnionHashSet::new_from([])).partial_cmp(&B::new(SetUnionHashSet::new_from([0]))), Some(Less));
        assert_eq!(B::new(SetUnionHashSet::new_from([0])).partial_cmp(&B::new(SetUnionHashSet::new_from([1]))), None);

        assert!(B::default().eq(&B::default()));
        assert!(!B::new(SetUnionHashSet::new_from([])).eq(&B::default()));
        assert!(!B::default().eq(&B::new_from(SetUnionHashSet::new_from([]))));
        assert!(B::new(SetUnionHashSet::new_from([])).eq(&B::new(SetUnionHashSet::new_from([]))));
        assert!(!B::new(SetUnionHashSet::new_from([0])).eq(&B::new(SetUnionHashSet::new_from([]))));
        assert!(!B::new(SetUnionHashSet::new_from([])).eq(&B::new(SetUnionHashSet::new_from([0]))));
        assert!(!B::new(SetUnionHashSet::new_from([0])).eq(&B::new(SetUnionHashSet::new_from([1]))));
    }

    #[test]
    fn consistency() {
        check_all(&[
            Top::default(),
            Top::new(SetUnionHashSet::new_from([])),
            Top::new(SetUnionHashSet::new_from([0])),
            Top::new(SetUnionHashSet::new_from([1])),
            Top::new(SetUnionHashSet::new_from([0, 1])),
        ])
    }
}
