//! Dominating pair compound lattice.
//!
//! When merging if one `Key` (usually a timestamp) fully dominates (is greater than) the other,
//! than both that `Key` and corresponding `Val` are selected. If the `Key`s are equal or
//! incomparable, then both the `Key`s and `Val`s are merged.

use std::cmp::Ordering;

use super::{Compare, ConvertFrom, Merge};

/// Dominating pair lattice.
///
/// `Key` specifies the key lattice (usually a timestamp), and `Val` specifies the value lattice.
pub struct DomPair<Key, Val> {
    key: Key,
    val: Val,
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
                self.key.merge(other.key);
                self.val.merge(other.val);
                true
            }
            Some(Ordering::Equal) => self.val.merge(other.val),
            Some(Ordering::Less) => {
                *self = DomPair {
                    key: ConvertFrom::from(other.key),
                    val: ConvertFrom::from(other.val),
                };
                true
            }
            Some(Ordering::Greater) => false,
        }
    }
}
