#![deny(missing_docs)]
#![feature(impl_trait_in_assoc_type)]
//! Module containing lattice traits and implementations.
//!
//! Convention: Generic parameters that are full words (e.g. `Other`, `Key`, `Val`) are lattice
//! types.
//! Conversely, Generic parameters that are single letters or acronyms (e.g. `K`, `T`) are scalar
//! non-`Lattice` types.

use std::cmp::Ordering;

use sealed::sealed;

pub mod bottom;
pub mod collections;
pub mod dom_pair;
pub mod fake;
pub mod map_union;
pub mod ord;
pub mod pair;
pub mod set_union;
pub mod tag;

/// Trait for lattice merge (least upper bound).
pub trait Merge<Other> {
    /// Merge `other` into the `self` lattice.
    ///
    /// Returns whether `self` changed at all.
    fn merge(&mut self, other: Other) -> bool;

    /// Merge `this` and `delta` together, returning the new value.
    fn merge_owned(mut this: Self, delta: Other) -> Self
    where
        Self: Sized,
    {
        Self::merge(&mut this, delta);
        this
    }
}

/// Naive lattice compare, based on the [`Merge::merge`] function.
#[sealed]
pub trait NaiveCompare<Other>
where
    Self: Clone + Merge<Other> + Sized,
    Other: Clone + Merge<Self>,
{
    /// Naive compare based on the [`Merge::merge`] method. This method can be very inefficient;
    /// use [`Compare::compare`] instead.
    ///
    /// This method should not be overridden.
    fn naive_compare(&self, other: &Other) -> Option<Ordering> {
        let mut self_a = self.clone();
        let other_a = other.clone();
        let self_b = self.clone();
        let mut other_b = other.clone();
        match (self_a.merge(other_a), other_b.merge(self_b)) {
            (true, true) => None,
            (true, false) => Some(Ordering::Less),
            (false, true) => Some(Ordering::Greater),
            (false, false) => Some(Ordering::Equal),
        }
    }
}
#[sealed]
impl<This, Other> NaiveCompare<Other> for This
where
    Self: Clone + Merge<Other>,
    Other: Clone + Merge<Self>,
{
}

/// Compare the partial order of two lattices.
///
/// Same signature as [`std::cmp::PartialOrd`] but without the connotation and also without the
/// [`std::cmp::PartialEq`] requirement.
pub trait Compare<Other> {
    /// Compare the partial order of self with other.
    ///
    /// `Some(Ordering::Less)` means `self` is less than `other`.
    fn compare(&self, other: &Other) -> Option<Ordering>;
}

/// Same as `From` but for lattices.
///
/// Do not convert non-lattice (AKA scalar) types if you implement this trait.
pub trait ConvertFrom<Other> {
    /// Convert from the `Other` lattice into `Self`.
    fn from(other: Other) -> Self;
}
