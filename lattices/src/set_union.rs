//! Set-union compound lattice.
//!
//! Merging set-union lattices is done by unioning the keys.

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};

use crate::cc_traits::{Iter, Len, Set};
use crate::collections::{ArraySet, SingletonSet};
use crate::{Compare, ConvertFrom, Merge};

/// A set-union lattice.
///
/// `Tag` specifies what datastructure to use, allowing us to deal with different datastructures
/// generically.
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SetUnion<Set>(pub Set);
impl<Set> SetUnion<Set> {
    /// Create a new `SetUnion` from a `Set`.
    pub fn new(val: Set) -> Self {
        Self(val)
    }

    /// Create a new `SetUnion` from an `Into<Set>`.
    pub fn new_from(val: impl Into<Set>) -> Self {
        Self::new(val.into())
    }
}

impl<SetSelf, SetOther, Item> Merge<SetUnion<SetOther>> for SetUnion<SetSelf>
where
    SetSelf: Extend<Item> + Len,
    SetOther: IntoIterator<Item = Item>,
{
    fn merge(&mut self, other: SetUnion<SetOther>) -> bool {
        let old_len = self.0.len();
        self.0.extend(other.0);
        self.0.len() > old_len
    }
}

impl<SetSelf, SetOther, Item> ConvertFrom<SetUnion<SetOther>> for SetUnion<SetSelf>
where
    SetSelf: FromIterator<Item>,
    SetOther: IntoIterator<Item = Item>,
{
    fn from(other: SetUnion<SetOther>) -> Self {
        Self(other.0.into_iter().collect())
    }
}

impl<SetSelf, SetOther, Item> Compare<SetUnion<SetOther>> for SetUnion<SetSelf>
where
    SetSelf: Set<Item, Item = Item> + Iter,
    SetOther: Set<Item, Item = Item> + Iter,
{
    fn compare(&self, other: &SetUnion<SetOther>) -> Option<Ordering> {
        match self.0.len().cmp(&other.0.len()) {
            Ordering::Greater => {
                if other.0.iter().all(|key| self.0.contains(&*key)) {
                    Some(Ordering::Greater)
                } else {
                    None
                }
            }
            Ordering::Equal => {
                if self.0.iter().all(|key| other.0.contains(&*key)) {
                    Some(Ordering::Equal)
                } else {
                    None
                }
            }
            Ordering::Less => {
                if self.0.iter().all(|key| other.0.contains(&*key)) {
                    Some(Ordering::Less)
                } else {
                    None
                }
            }
        }
    }
}

/// [`std::collections::HashSet`]-backed [`SetUnion`] lattice.
pub type SetUnionHashSet<Item> = SetUnion<HashSet<Item>>;

/// [`std::collections::BTreeSet`]-backed [`SetUnion`] lattice.
pub type SetUnionBTreeSet<Item> = SetUnion<BTreeSet<Item>>;

/// [`Vec`]-backed [`SetUnion`] lattice.
pub type SetUnionVec<Item> = SetUnion<Vec<Item>>;

/// [`crate::collections::ArraySet`]-backed [`SetUnion`] lattice.
pub type SetUnionArray<Item, const N: usize> = SetUnion<ArraySet<Item, N>>;

/// [`crate::collections::SingletonSet`]-backed [`SetUnion`] lattice.
pub type SetUnionSingletonSet<Item> = SetUnion<SingletonSet<Item>>;

/// [`Option`]-backed [`SetUnion`] lattice.
pub type SetUnionOption<Item> = SetUnion<Option<Item>>;

#[cfg(test)]
mod test {
    use super::*;

    use crate::collections::SingletonSet;

    #[test]
    fn test_set_union() {
        let mut my_set_a = SetUnionHashSet::<&str>::new(HashSet::new());
        let my_set_b = SetUnionBTreeSet::<&str>::new(BTreeSet::new());
        let my_set_c = SetUnionSingletonSet::new(SingletonSet("hello world"));

        assert_eq!(Some(Ordering::Equal), my_set_a.compare(&my_set_a));
        assert_eq!(Some(Ordering::Equal), my_set_a.compare(&my_set_b));
        assert_eq!(Some(Ordering::Less), my_set_a.compare(&my_set_c));
        assert_eq!(Some(Ordering::Equal), my_set_b.compare(&my_set_a));
        assert_eq!(Some(Ordering::Equal), my_set_b.compare(&my_set_b));
        assert_eq!(Some(Ordering::Less), my_set_b.compare(&my_set_c));
        assert_eq!(Some(Ordering::Greater), my_set_c.compare(&my_set_a));
        assert_eq!(Some(Ordering::Greater), my_set_c.compare(&my_set_b));
        assert_eq!(Some(Ordering::Equal), my_set_c.compare(&my_set_c));

        my_set_a.compare(&my_set_b);
        my_set_a.compare(&my_set_c);
        my_set_b.compare(&my_set_c);

        my_set_a.merge(my_set_b);
        my_set_a.merge(my_set_c);
    }

    #[test]
    fn test_singleton_example() {
        let mut my_hash_set = SetUnionHashSet::<&str>::default();
        let my_delta_set = SetUnionSingletonSet::new_from("hello world");
        let my_array_set = SetUnionArray::new_from(["hello world", "b", "c", "d"]);

        assert_eq!(Some(Ordering::Equal), my_delta_set.compare(&my_delta_set));
        assert_eq!(Some(Ordering::Less), my_delta_set.compare(&my_array_set));
        assert_eq!(Some(Ordering::Greater), my_array_set.compare(&my_delta_set));
        assert_eq!(Some(Ordering::Equal), my_array_set.compare(&my_array_set));

        assert!(my_hash_set.merge(my_array_set)); // Changes
        assert!(!my_hash_set.merge(my_delta_set)); // No changes

        println!("{:?}", my_hash_set.0);
    }
}
