#![deny(missing_docs)]
//! Module containing lattice traits and implementations.
//!
//! Convention: Generic parameters that are full words (e.g. `Other`, `Key`, `Val`) are lattice
//! types.
//! Conversely, Generic parameters that are single letters or acronyms (e.g. `K`, `T`) are scalar
//! non-`Lattice` types.
use std::cmp::Ordering;

pub mod dom_pair;
pub mod map_union;
pub mod ord;
pub mod set_union;

/// Trait for lattice merge (least upper bound).
pub trait Merge<Other> {
    /// Merge `other` into the `self` lattice.
    fn merge(&mut self, other: Other) -> bool;
}

/// Same as `From` but for lattices.
///
/// Do not convert non-lattice (AKA scalar) types if you implement this trait.
pub trait ConvertFrom<Other> {
    /// Convert from the `Other` lattice into `Self`.
    fn from(other: Other) -> Self;
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
