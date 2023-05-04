//! Totally-ordered lattices, [`Max`] and [`Min`].
//!
//! Uses [std::cmp::Ord`].

use super::{Compare, ConvertFrom, Merge};

/// A totally ordered max lattice. Merging takes the larger value.
#[repr(transparent)]
#[derive(Default, PartialEq, PartialOrd, Eq, Ord, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Max<T>(pub T);
impl<T> Max<T> {
    /// Create a new `Max` lattice instance from a `T`.
    pub fn new(val: T) -> Self {
        Self(val)
    }

    /// Create a new `Max` lattice instance from an `Into<T>` value.
    pub fn from(val: impl Into<T>) -> Self {
        Self::new(val.into())
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

impl<T> Compare<Max<T>> for Max<T>
where
    T: Ord,
{
    fn compare(&self, other: &Max<T>) -> Option<std::cmp::Ordering> {
        Some(Ord::cmp(&self.0, &other.0))
    }
}

/// A totally ordered min lattice. Merging takes the smaller value.
#[repr(transparent)]
#[derive(Default, PartialEq, PartialOrd, Eq, Ord, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Min<T>(pub T);
impl<T> Min<T> {
    /// Create a new `Min` lattice instance from a `T`.
    pub fn new(val: T) -> Self {
        Self(val)
    }

    /// Create a new `Min` lattice instance from an `Into<T>` value.
    pub fn new_from(val: impl Into<T>) -> Self {
        Self::new(val.into())
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
        Some(Ord::cmp(&self.0, &other.0).reverse())
    }
}
