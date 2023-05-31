use super::{ConvertFrom, Merge};
use crate::LatticeOrd;

/// A `Immut` lattice that will runtime panic if a merge between inequal values is attempted.
///
/// This is used to wrap non lattice data into a lattice in a way that typechecks
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Immut<T>(pub T);
impl<T> Immut<T> {
    /// Create a new `Immut` lattice instance from a value.
    pub fn new(val: T) -> Self {
        Self(val)
    }

    /// Create a new `Immut` lattice instance from a value using `Into`.
    pub fn new_from(val: impl Into<T>) -> Self {
        Self::new(val.into())
    }
}

impl<T, O> Merge<Immut<O>> for Immut<T>
where
    T: PartialEq<O>,
{
    fn merge(&mut self, other: Immut<O>) -> bool {
        if self.0 != other.0 {
            panic!("The `Immut` lattice cannot merge inequal elements.")
        }
        false
    }
}

impl<T> ConvertFrom<Immut<T>> for Immut<T> {
    fn from(other: Immut<T>) -> Self {
        other
    }
}

impl<T, O> PartialOrd<Immut<O>> for Immut<T>
where
    T: PartialEq<O>,
{
    fn partial_cmp(&self, other: &Immut<O>) -> Option<std::cmp::Ordering> {
        if self.0 != other.0 {
            panic!("The `Immut` lattice does not have a partial order between inequal elements.");
        }
        Some(std::cmp::Ordering::Equal)
    }
}
impl<T, O> LatticeOrd<Immut<O>> for Immut<T> where Self: PartialOrd<Immut<O>> {}

impl<T, O> PartialEq<Immut<O>> for Immut<T>
where
    T: PartialEq<O>,
{
    fn eq(&self, other: &Immut<O>) -> bool {
        self.0 == other.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::{
        check_all, check_lattice_ord, check_lattice_properties, check_partial_ord_properties,
    };

    #[test]
    fn consistency_equal() {
        check_all(&[Immut::new("hello world")])
    }

    #[test]
    fn consistency_inequal() {
        use std::collections::BTreeSet;

        let items = [
            Immut::new(BTreeSet::from_iter([])),
            Immut::new(BTreeSet::from_iter([0])),
            Immut::new(BTreeSet::from_iter([1])),
            Immut::new(BTreeSet::from_iter([0, 1])),
        ];

        // Merged inequal elements panic, therefore `NaiveMerge` panics.
        assert!(std::panic::catch_unwind(|| check_lattice_ord(&items)).is_err());
        // `Immut` does not have a partial order.
        assert!(std::panic::catch_unwind(|| check_partial_ord_properties(&items)).is_err());
        // `Immut` is not actually a lattice.
        assert!(std::panic::catch_unwind(|| check_lattice_properties(&items)).is_err());
    }
}
