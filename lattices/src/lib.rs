#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use std::cmp::Ordering::{self, *};

pub use cc_traits;
use sealed::sealed;

pub mod collections;
mod conflict;
mod dom_pair;
pub mod map_union;
mod ord;
mod pair;
mod point;
pub mod set_union;
pub mod test;
mod unit;
mod vec_union;
mod with_bot;
mod with_top;

pub use conflict::Conflict;
pub use dom_pair::DomPair;
pub use ord::{Max, Min};
pub use pair::Pair;
pub use point::Point;
pub use vec_union::VecUnion;
pub use with_bot::WithBot;
pub use with_top::WithTop;

/// Trait for lattice merge (AKA "join" or "least upper bound").
pub trait Merge<Other> {
    /// Merge `other` into the `self` lattice.
    ///
    /// This operation must be associative, commutative, and idempotent.
    ///
    /// Returns `true` if `self` changed, `false` otherwise.
    /// Returning `true` implies that the new value for `self` is later in the lattice order than
    /// the old value. Returning `false` means that `self` was unchanged and therefore `other` came
    /// before `self` (or the two are equal) in the lattice order.
    fn merge(&mut self, other: Other) -> bool;

    /// Merge `this` and `delta` together, returning the new value.
    fn merge_owned(mut self, delta: Other) -> Self
    where
        Self: Sized,
    {
        Self::merge(&mut self, delta);
        self
    }
}

/// Trait for lattice partial order comparison
/// PartialOrd is implemented for many things, this trait can be used to require the type be a lattice.
pub trait LatticeOrd<Rhs = Self>: PartialOrd<Rhs> {}

/// Naive lattice compare, based on the [`Merge::merge`] function.
#[sealed]
pub trait NaiveLatticeOrd<Rhs = Self>
where
    Self: Clone + Merge<Rhs> + Sized,
    Rhs: Clone + Merge<Self>,
{
    /// Naive compare based on the [`Merge::merge`] method. This method can be very inefficient;
    /// use [`PartialOrd::partial_cmp`] instead.
    ///
    /// This method should not be overridden.
    fn naive_cmp(&self, other: &Rhs) -> Option<Ordering> {
        let mut self_a = self.clone();
        let other_a = other.clone();
        let self_b = self.clone();
        let mut other_b = other.clone();
        match (self_a.merge(other_a), other_b.merge(self_b)) {
            (true, true) => None,
            (true, false) => Some(Less),
            (false, true) => Some(Greater),
            (false, false) => Some(Equal),
        }
    }
}
#[sealed]
impl<This, Other> NaiveLatticeOrd<Other> for This
where
    Self: Clone + Merge<Other>,
    Other: Clone + Merge<Self>,
{
}

/// Same as `From` but for lattices.
///
/// This should only be implemented between different representations of the same lattice type.
/// This should recursively convert nested lattice types, but not non-lattice ("scalar") types.
pub trait LatticeFrom<Other> {
    /// Convert from the `Other` lattice into `Self`.
    fn lattice_from(other: Other) -> Self;
}

/// Trait to check if a lattice instance is bottom (⊥).
pub trait IsBot {
    /// Returns if `self` is lattice bottom (⊥).
    fn is_bot(&self) -> bool;
}

/// Trait to check if a lattice instance is top (⊤) and therefore cannot change any futher.
pub trait IsTop {
    /// Returns if `self` is lattice top (⊤).
    fn is_top(&self) -> bool;
}

/// Trait for "un-merging" (subtracting or differentiating) lattices.
///
/// For `A unmerge B`:
/// * `A > B`: In the simplest case, this acts as the inverse to [`Merge`]. This simplest case
/// occurs only when the instances being subtracted is _strictly less than_ the instance being
/// unmerged from. Mathematically, if `Z = A unmerge B` then `B merge Z = A`. If multiple values
/// satisfy this condition, `Z` should be the smallest of such values, if a smallest value exists.
/// * `A <= B`: In this case the instance being subtracted is greater than or equal the original.
/// The result is bottom, and `true` is returned (unless `A` is already bottom, in which case `false`)
/// is returned because it is unchanged.
/// * `A` and `B` are incomparable. In this case, we logically substitute `B` for a different value
/// `B'` which _is_ in the past of `A`: `A > B'`. `B'` is defined to be the largest value which is
/// less than both `A` and `B`, the Greatest Lower Bound (GLB), also called the _meet_. Then
/// subtract `B'` from `A` as in the simplest case above.
pub trait Unmerge<Other> {
    /// "Un-merges" `other` from `self`, roughly `self = self - other`.
    ///
    /// Returns `false` without modifying `self` if `other > self`. This can be thought of as
    /// returning bottom (regardless of if bottom actually exists for `Self`).
    fn unmerge(&mut self, other: &Other) -> bool;

    /// "Un-merges" `other` from `self`, roughly `self - other`.
    ///
    /// Returns `None` if `other > self`. This can be thought of as returning bottom (regardless
    /// of if bottom actually exists for `Self`).
    fn unmerge_owned(mut self, other: &Other) -> Option<Self>
    where
        Self: Sized,
    {
        Self::unmerge(&mut self, other).then_some(self)
    }
}

/// Trait to atomize a lattice into individual elements. For example, a [`set_union::SetUnion`]
/// will be broken up into individual singleton elements.
///
/// Formally, breaks up `Self` into an set of lattice points forming a (strong) [antichain](https://en.wikipedia.org/wiki/Antichain).
pub trait Atomize: Merge<Self::Atom> {
    /// The type of atoms for this lattice.
    type Atom: 'static;

    /// The iter type iterating the antichain atoms.
    type AtomIter: 'static + Iterator<Item = Self::Atom>;

    /// Atomize self: convert into an iter of atoms.
    ///
    /// Must always return at least one value.
    ///
    /// Returned values must merge to reform a value equal to the original `self`.
    fn atomize(self) -> Self::AtomIter;
}
