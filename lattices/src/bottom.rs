//! Gives a default representation that compares as less than to a lattice.
//!
//! This can be used for giving a sensible default repersentation to types that don't necessarily have one.

use std::cmp::Ordering::{self, *};

use super::{ConvertFrom, Merge};

/// Bottom wrapper.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bottom<Inner>(pub Option<Inner>);
impl<Inner> Bottom<Inner> {
    /// Create a new `Bottom` lattice instance from a value.
    pub fn new(val: Inner) -> Self {
        Self(Some(val))
    }

    /// Create a new `Bottom` lattice instance from a value using `Into`.
    pub fn new_from(val: impl Into<Inner>) -> Self {
        Self::new(val.into())
    }
}

impl<Inner, Other> Merge<Bottom<Other>> for Bottom<Inner>
where
    Inner: Merge<Other> + ConvertFrom<Other>,
{
    fn merge(&mut self, other: Bottom<Other>) -> bool {
        match (&mut self.0, other.0) {
            (None, None) => false,
            (Some(_), None) => false,
            (this @ None, Some(other_inner)) => {
                *this = Some(ConvertFrom::from(other_inner));
                true
            }
            (Some(self_inner), Some(other_inner)) => self_inner.merge(other_inner),
        }
    }
}

impl<Inner> ConvertFrom<Bottom<Inner>> for Bottom<Inner> {
    fn from(other: Bottom<Inner>) -> Self {
        other
    }
}

impl<Inner, Other> PartialOrd<Bottom<Other>> for Bottom<Inner>
where
    Inner: PartialOrd<Other>,
{
    fn partial_cmp(&self, other: &Bottom<Other>) -> Option<Ordering> {
        match (&self.0, &other.0) {
            (None, None) => Some(Equal),
            (None, Some(_)) => Some(Less),
            (Some(_), None) => Some(Greater),
            (Some(this_inner), Some(other_inner)) => this_inner.partial_cmp(other_inner),
        }
    }
}

impl<Inner, Other> PartialEq<Bottom<Other>> for Bottom<Inner>
where
    Inner: PartialEq<Other>,
{
    fn eq(&self, other: &Bottom<Other>) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => false,
            (Some(this_inner), Some(other_inner)) => this_inner == other_inner,
        }
    }
}
impl<Inner> Eq for Bottom<Inner> where Inner: PartialEq {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::set_union::SetUnionHashSet;
    use crate::test::{assert_lattice_identities, assert_partial_ord_identities};

    #[test]
    fn consistency() {
        let test_vec = vec![
            Bottom::default(),
            Bottom::new(SetUnionHashSet::new_from([])),
            Bottom::new(SetUnionHashSet::new_from([0])),
            Bottom::new(SetUnionHashSet::new_from([1])),
            Bottom::new(SetUnionHashSet::new_from([0, 1])),
        ];

        assert_partial_ord_identities(&test_vec);
        assert_lattice_identities(&test_vec);
    }
}
