//! A fake lattice that will runtime panic if a merge is attempted.
//!
//! This is used to wrap non lattice data into a lattice in a way that typechecks

use super::{ConvertFrom, Merge};

#[repr(transparent)]
#[derive(Debug, Clone)]
/// Fake lattice.
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

impl<T> Merge<Fake<T>> for Fake<T> {
    fn merge(&mut self, _: Fake<T>) -> bool {
        panic!("The fake lattice cannot be merged.")
    }
}

impl<T> ConvertFrom<Fake<T>> for Fake<T> {
    fn from(other: Fake<T>) -> Self {
        other
    }
}

#[cfg(test)]
mod test {
    use super::*;

    type Fake = super::Fake<usize>;

    #[test]
    #[should_panic]
    fn merge() {
        Fake::new(7usize).merge(Fake::new(7usize));
    }
}
