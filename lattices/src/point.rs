use super::{LatticeFrom, Merge};
use crate::LatticeOrd;

/// A `Point` lattice, corresponding to a single instance of `T`.
///
/// Will runtime panic if a merge between inequal values is attempted.
///
/// Can be thought of as a lattice with a domain of size one, corresponding to the specific value
/// inside.
///
/// This also can be used to wrap non lattice data into a lattice in a way that typechecks.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point<T>(pub T);
impl<T> Point<T> {
    /// Create a new `Point` lattice instance from a value.
    pub fn new(val: T) -> Self {
        Self(val)
    }

    /// Create a new `Point` lattice instance from a value using `Into`.
    pub fn new_from(val: impl Into<T>) -> Self {
        Self::new(val.into())
    }
}

impl<T, O> Merge<Point<O>> for Point<T>
where
    T: PartialEq<O>,
{
    fn merge(&mut self, other: Point<O>) -> bool {
        if self.0 != other.0 {
            panic!("The `Point` lattice cannot merge inequal elements.")
        }
        false
    }
}

impl<T> LatticeFrom<Point<T>> for Point<T> {
    fn lattice_from(other: Point<T>) -> Self {
        other
    }
}

impl<T, O> PartialOrd<Point<O>> for Point<T>
where
    T: PartialEq<O>,
{
    fn partial_cmp(&self, other: &Point<O>) -> Option<std::cmp::Ordering> {
        if self.0 != other.0 {
            panic!("The `Point` lattice does not have a partial order between inequal elements.");
        }
        Some(std::cmp::Ordering::Equal)
    }
}
impl<T, O> LatticeOrd<Point<O>> for Point<T> where Self: PartialOrd<Point<O>> {}

impl<T, O> PartialEq<Point<O>> for Point<T>
where
    T: PartialEq<O>,
{
    fn eq(&self, other: &Point<O>) -> bool {
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
        check_all(&[Point::new("hello world")])
    }

    #[test]
    fn consistency_inequal() {
        use std::collections::BTreeSet;

        let items = [
            Point::new(BTreeSet::from_iter([])),
            Point::new(BTreeSet::from_iter([0])),
            Point::new(BTreeSet::from_iter([1])),
            Point::new(BTreeSet::from_iter([0, 1])),
        ];

        // Merged inequal elements panic, therefore `NaiveMerge` panics.
        assert!(std::panic::catch_unwind(|| check_lattice_ord(&items)).is_err());
        // `Point` does not have a partial order.
        assert!(std::panic::catch_unwind(|| check_partial_ord_properties(&items)).is_err());
        // `Point` is not actually a lattice.
        assert!(std::panic::catch_unwind(|| check_lattice_properties(&items)).is_err());
    }
}
