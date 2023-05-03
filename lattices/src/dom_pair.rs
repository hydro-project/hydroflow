//! Dominating pair compound lattice.
//!
//! When merging if one `Key` (usually a timestamp) fully dominates (is greater than) the other,
//! then both that `Key` and corresponding `Val` are selected. If the `Key`s are equal or
//! incomparable, then both the `Key`s and `Val`s are merged.

use std::cmp::Ordering;

use super::{Compare, ConvertFrom, Merge};

/// Dominating pair lattice.
///
/// `Key` specifies the key lattice (usually a timestamp), and `Val` specifies the value lattice.
#[derive(Clone, Debug)]
pub struct DomPair<Key, Val> {
    /// The `Key` of the  dominating pair lattice, usually a timestamp
    pub key: Key,
    /// The `Val` of the dominating pair lattice.
    pub val: Val,
}

impl<Key, Val> DomPair<Key, Val> {
    /// Create a `DomPair` from the given `Key` and `Val`.
    pub fn new(key: Key, val: Val) -> Self {
        Self { key, val }
    }

    /// Create a `DomPair` from the given `Into<Key>` and `Into<Val>`.
    pub fn new_from(key: impl Into<Key>, val: impl Into<Val>) -> Self {
        Self::new(key.into(), val.into())
    }
}

impl<KeySelf, KeyOther, ValSelf, ValOther> Merge<DomPair<KeyOther, ValOther>>
    for DomPair<KeySelf, ValSelf>
where
    KeySelf: Merge<KeyOther> + ConvertFrom<KeyOther> + Compare<KeyOther>,
    ValSelf: Merge<ValOther> + ConvertFrom<ValOther>,
{
    fn merge(&mut self, other: DomPair<KeyOther, ValOther>) -> bool {
        match self.key.compare(&other.key) {
            None => {
                assert!(self.key.merge(other.key));
                self.val.merge(other.val);
                true
            }
            Some(Ordering::Equal) => self.val.merge(other.val),
            Some(Ordering::Less) => {
                *self = ConvertFrom::from(other);
                true
            }
            Some(Ordering::Greater) => false,
        }
    }
}

impl<KeySelf, KeyOther, ValSelf, ValOther> ConvertFrom<DomPair<KeyOther, ValOther>>
    for DomPair<KeySelf, ValSelf>
where
    KeySelf: ConvertFrom<KeyOther>,
    ValSelf: ConvertFrom<ValOther>,
{
    fn from(other: DomPair<KeyOther, ValOther>) -> Self {
        Self {
            key: ConvertFrom::from(other.key),
            val: ConvertFrom::from(other.val),
        }
    }
}

impl<KeySelf, KeyOther, ValSelf, ValOther> Compare<DomPair<KeyOther, ValOther>>
    for DomPair<KeySelf, ValSelf>
where
    KeySelf: Compare<KeyOther>,
    ValSelf: Compare<ValOther>,
{
    fn compare(&self, other: &DomPair<KeyOther, ValOther>) -> Option<Ordering> {
        match self.key.compare(&other.key) {
            Some(Ordering::Equal) => self.val.compare(&other.val),
            otherwise => otherwise,
        }
    }
}

impl<Key, Val> Default for DomPair<Key, Val>
where
    Key: Default,
    Val: Default,
{
    fn default() -> Self {
        let (key, val) = Default::default();
        Self { key, val }
    }
}
