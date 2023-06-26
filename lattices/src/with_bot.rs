use std::cmp::Ordering::{self, *};

use crate::{IsBot, IsTop, LatticeFrom, LatticeOrd, Merge};

/// Wraps a lattice in [`Option`], treating [`None`] as a new bottom element which compares as less
/// than to all other values.
///
/// This can be used for giving a sensible default/bottom element to lattices that don't
/// necessarily have one.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WithBot<Inner>(pub Option<Inner>);
impl<Inner> WithBot<Inner> {
    /// Create a new `WithBot` lattice instance from a value.
    pub fn new(val: Option<Inner>) -> Self {
        Self(val)
    }

    /// Create a new `WithBot` lattice instance from a value using `Into`.
    pub fn new_from(val: impl Into<Option<Inner>>) -> Self {
        Self::new(val.into())
    }
}

// Cannot auto derive because the generated implementation has the wrong trait bounds.
// https://github.com/rust-lang/rust/issues/26925
impl<Inner> Default for WithBot<Inner> {
    fn default() -> Self {
        Self(None)
    }
}

impl<Inner, Other> Merge<WithBot<Other>> for WithBot<Inner>
where
    Inner: Merge<Other> + LatticeFrom<Other>,
{
    fn merge(&mut self, other: WithBot<Other>) -> bool {
        match (&mut self.0, other.0) {
            (None, None) => false,
            (Some(_), None) => false,
            (this @ None, Some(other_inner)) => {
                *this = Some(LatticeFrom::lattice_from(other_inner));
                true
            }
            (Some(self_inner), Some(other_inner)) => self_inner.merge(other_inner),
        }
    }
}

impl<Inner, Other> LatticeFrom<WithBot<Other>> for WithBot<Inner>
where
    Inner: LatticeFrom<Other>,
{
    fn lattice_from(other: WithBot<Other>) -> Self {
        Self(other.0.map(Inner::lattice_from))
    }
}

impl<Inner, Other> PartialOrd<WithBot<Other>> for WithBot<Inner>
where
    Inner: PartialOrd<Other>,
{
    fn partial_cmp(&self, other: &WithBot<Other>) -> Option<Ordering> {
        match (&self.0, &other.0) {
            (None, None) => Some(Equal),
            (None, Some(_)) => Some(Less),
            (Some(_), None) => Some(Greater),
            (Some(this_inner), Some(other_inner)) => this_inner.partial_cmp(other_inner),
        }
    }
}
impl<Inner, Other> LatticeOrd<WithBot<Other>> for WithBot<Inner> where
    Self: PartialOrd<WithBot<Other>>
{
}

impl<Inner, Other> PartialEq<WithBot<Other>> for WithBot<Inner>
where
    Inner: PartialEq<Other>,
{
    fn eq(&self, other: &WithBot<Other>) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => false,
            (Some(this_inner), Some(other_inner)) => this_inner == other_inner,
        }
    }
}

impl<Inner> IsBot for WithBot<Inner> {
    fn is_bot(&self) -> bool {
        self.0.is_none()
    }
}

impl<Inner> IsTop for WithBot<Inner>
where
    Inner: IsTop,
{
    fn is_top(&self) -> bool {
        self.0.as_ref().map_or(false, IsTop::is_top)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::set_union::{SetUnionHashSet, SetUnionSingletonSet};
    use crate::test::check_all;

    #[test]
    fn test_singly_nested_singleton_example() {
        let mut my_hash_set = WithBot::new_from(SetUnionHashSet::<&str>::default());
        let my_delta_set = WithBot::new_from(SetUnionSingletonSet::new_from("hello world"));

        assert!(my_hash_set.merge(my_delta_set)); // Changes
        assert!(!my_hash_set.merge(my_delta_set)); // No changes
    }

    #[test]
    fn test_doubly_nested_singleton_example() {
        let mut my_hash_set =
            WithBot::new_from(WithBot::new_from(SetUnionHashSet::<&str>::default()));
        let my_delta_set = WithBot::new_from(WithBot::new_from(SetUnionSingletonSet::new_from(
            "hello world",
        )));

        assert!(my_hash_set.merge(my_delta_set)); // Changes
        assert!(!my_hash_set.merge(my_delta_set)); // No changes
    }

    #[test]
    #[rustfmt::skip]
    fn auto_derives() {
        type B = WithBot<SetUnionHashSet<usize>>;

        assert_eq!(B::default().partial_cmp(&B::default()), Some(Equal));
        assert_eq!(B::new_from(SetUnionHashSet::new_from([])).partial_cmp(&B::default()), Some(Greater));
        assert_eq!(B::default().partial_cmp(&B::new_from(SetUnionHashSet::new_from([]))), Some(Less));
        assert_eq!(B::new_from(SetUnionHashSet::new_from([])).partial_cmp(&B::new_from(SetUnionHashSet::new_from([]))), Some(Equal));
        assert_eq!(B::new_from(SetUnionHashSet::new_from([0])).partial_cmp(&B::new_from(SetUnionHashSet::new_from([]))), Some(Greater));
        assert_eq!(B::new_from(SetUnionHashSet::new_from([])).partial_cmp(&B::new_from(SetUnionHashSet::new_from([0]))), Some(Less));
        assert_eq!(B::new_from(SetUnionHashSet::new_from([0])).partial_cmp(&B::new_from(SetUnionHashSet::new_from([1]))), None);

        assert!(B::default().eq(&B::default()));
        assert!(!B::new_from(SetUnionHashSet::new_from([])).eq(&B::default()));
        assert!(!B::default().eq(&B::new_from(SetUnionHashSet::new_from([]))));
        assert!(B::new_from(SetUnionHashSet::new_from([])).eq(&B::new_from(SetUnionHashSet::new_from([]))));
        assert!(!B::new_from(SetUnionHashSet::new_from([0])).eq(&B::new_from(SetUnionHashSet::new_from([]))));
        assert!(!B::new_from(SetUnionHashSet::new_from([])).eq(&B::new_from(SetUnionHashSet::new_from([0]))));
        assert!(!B::new_from(SetUnionHashSet::new_from([0])).eq(&B::new_from(SetUnionHashSet::new_from([1]))));
    }

    #[test]
    fn consistency() {
        check_all(&[
            WithBot::default(),
            WithBot::new_from(SetUnionHashSet::new_from([])),
            WithBot::new_from(SetUnionHashSet::new_from([0])),
            WithBot::new_from(SetUnionHashSet::new_from([1])),
            WithBot::new_from(SetUnionHashSet::new_from([0, 1])),
        ])
    }
}
