use std::cmp::Ordering::{self, *};

use crate::{IsTop, LatticeFrom, LatticeOrd, Merge};

/// A `Conflict` lattice, stores a single instance of `T` and goes to a "conflict" state (`None`)
/// if inequal `T` instances are merged together.
///
/// Like [`Point<T>`](crate::Point), but will go to "conflict" (top/`None`) instead of panicking.
///
/// Can be thought of as a lattice with a domain of size one, corresponding to the specific value
/// inside.
///
/// This can be used to wrap non-lattice (scalar) data into a lattice type.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Conflict<T>(pub Option<T>);
impl<T> Conflict<T> {
    /// Create a new `Conflict` lattice instance from a value.
    pub fn new(val: Option<T>) -> Self {
        Self(val)
    }

    /// Create a new `Conflict` lattice instance from a value using `Into`.
    pub fn new_from(val: impl Into<Option<T>>) -> Self {
        Self::new(val.into())
    }
}

impl<T, O> Merge<Conflict<O>> for Conflict<T>
where
    T: PartialEq<O>,
{
    fn merge(&mut self, other: Conflict<O>) -> bool {
        if let Some(val_self) = &self.0 {
            if other.0.map_or(true, |val_other| val_self != &val_other) {
                self.0 = None;
                return true;
            }
        }
        false
    }
}

impl<T> LatticeFrom<Conflict<T>> for Conflict<T> {
    fn lattice_from(other: Conflict<T>) -> Self {
        other
    }
}

impl<T, O> PartialOrd<Conflict<O>> for Conflict<T>
where
    T: PartialEq<O>,
{
    fn partial_cmp(&self, other: &Conflict<O>) -> Option<Ordering> {
        match (&self.0, &other.0) {
            (None, None) => Some(Equal),
            (None, Some(_)) => Some(Greater),
            (Some(_), None) => Some(Less),
            (Some(val_self), Some(val_other)) => (val_self == val_other).then_some(Equal),
        }
    }
}
impl<T, O> LatticeOrd<Conflict<O>> for Conflict<T> where Self: PartialOrd<Conflict<O>> {}

impl<T, O> PartialEq<Conflict<O>> for Conflict<T>
where
    T: PartialEq<O>,
{
    fn eq(&self, other: &Conflict<O>) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(val_self), Some(val_other)) => val_self == val_other,
            _ => false,
        }
    }
}

impl<T> IsTop for Conflict<T> {
    fn is_top(&self) -> bool {
        self.0.is_none()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::{
        check_all, check_lattice_ord, check_lattice_properties, check_lattice_top,
        check_partial_ord_properties,
    };
    use crate::WithBot;

    #[test]
    fn consistency() {
        let items = &[
            Conflict::new_from("foo"),
            Conflict::new_from("bar"),
            Conflict::new(None),
        ];
        check_lattice_ord(items);
        check_partial_ord_properties(items);
        check_lattice_properties(items);
        // check_lattice_bot(items);
        check_lattice_top(items);
    }

    #[test]
    fn consistency_withbot() {
        let items = &[
            WithBot::new_from(Conflict::new_from("foo")),
            WithBot::new_from(Conflict::new_from("bar")),
            WithBot::new_from(Conflict::new(None)),
            WithBot::new(None),
        ];
        check_all(items);
        check_lattice_top(items);
    }
}
