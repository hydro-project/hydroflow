//! A fake lattice that will runtime panic if a merge is attempted.
//!
//! This is used to wrap non lattice data into a lattice in a way that typechecks

use super::{ConvertFrom, Merge};
use crate::LatticeOrd;

/// Fake lattice.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fake<T>(pub T);
impl<T> Fake<T> {
    /// Create a new `Fake` lattice instance from a value.
    pub fn new(val: T) -> Self {
        Self(val)
    }

    /// Create a new `Fake` lattice instance from a value using `Into`.
    pub fn new_from(val: impl Into<T>) -> Self {
        Self::new(val.into())
    }
}

impl<T, O> Merge<Fake<O>> for Fake<T> {
    fn merge(&mut self, _: Fake<O>) -> bool {
        panic!("The fake lattice cannot be merged.")
    }
}

impl<T> ConvertFrom<Fake<T>> for Fake<T> {
    fn from(other: Fake<T>) -> Self {
        other
    }
}

impl<T, O> PartialOrd<Fake<O>> for Fake<T>
where
    T: PartialOrd<O>,
{
    fn partial_cmp(&self, _: &Fake<O>) -> Option<std::cmp::Ordering> {
        panic!("The fake lattice does not have a partial order")
    }
}
impl<T, O> LatticeOrd<Fake<O>> for Fake<T> where Self: PartialOrd<Fake<O>> {}

impl<T, O> PartialEq<Fake<O>> for Fake<T>
where
    T: PartialEq<O>,
{
    fn eq(&self, other: &Fake<O>) -> bool {
        self.0 == other.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        set_union::SetUnionHashSet,
        test::{assert_lattice_identities, assert_partial_ord_identities},
    };

    #[test]
    fn consistency() {
        let test_vec = vec![
            Fake::new(SetUnionHashSet::new_from([])),
            Fake::new(SetUnionHashSet::new_from([0])),
            Fake::new(SetUnionHashSet::new_from([1])),
            Fake::new(SetUnionHashSet::new_from([0, 1])),
        ];

        // Fake does not have a partial order.
        assert!(std::panic::catch_unwind(|| assert_partial_ord_identities(&test_vec)).is_err());
        // Fake is not actually a lattice.
        assert!(std::panic::catch_unwind(|| assert_lattice_identities(&test_vec)).is_err());
    }
}
