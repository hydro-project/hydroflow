//! Totally-ordered lattices, [`Max`] and [`Min`].
//!
//! Uses [std::cmp::Ord`].

use super::{Compare, ConvertFrom, Merge};

#[repr(transparent)]
#[derive(Default, PartialEq, PartialOrd, Eq, Ord)]
/// A totally ordered max lattice. Merging takes the larger value.
pub struct Max<T>(T);
impl<T> Max<T> {
    /// Create a new `Max` lattice instance from a value.
    pub fn new(val: impl Into<T>) -> Self {
        Self(val.into())
    }
}

impl<T> Merge<Max<T>> for Max<T>
where
    T: Ord,
{
    fn merge(&mut self, other: Max<T>) -> bool {
        if self.0 < other.0 {
            self.0 = other.0;
            true
        } else {
            false
        }
    }
}

impl<T> ConvertFrom<Max<T>> for Max<T> {
    fn from(other: Max<T>) -> Self {
        other
    }
}

#[repr(transparent)]
#[derive(Default, PartialEq, PartialOrd, Eq, Ord)]
/// A totally ordered min lattice. Merging takes the smaller value.
pub struct Min<T>(pub T);
impl<T> Min<T> {
    /// Create a new `Min` lattice instance from a value.
    pub fn new(val: impl Into<T>) -> Self {
        Self(val.into())
    }
}

impl<T> Merge<Min<T>> for Min<T>
where
    T: Ord,
{
    fn merge(&mut self, other: Min<T>) -> bool {
        if other.0 < self.0 {
            self.0 = other.0;
            true
        } else {
            false
        }
    }
}

impl<T> ConvertFrom<Min<T>> for Min<T> {
    fn from(other: Min<T>) -> Self {
        other
    }
}

impl<T> Compare<Min<T>> for Min<T>
where
    T: Ord,
{
    fn compare(&self, other: &Min<T>) -> Option<std::cmp::Ordering> {
        Some(Ord::cmp(&self.0, &other.0))
    }
}
