//! Set-union compound lattice.
//!
//! Merging set-union lattices is done by unioning the keys.

use std::cmp::Ordering;

use crate::{collections::Collection, tag};

use super::{Compare, ConvertFrom, Merge};

/// A set-union lattice.
///
/// `Tag` specifies what datastructure to use, allowing us to deal with different datastructures
/// generically.
#[repr(transparent)]
pub struct SetUnion<Tag, T>(Tag::Bind)
where
    Tag: tag::Tag1<T>;
impl<Tag, T> SetUnion<Tag, T>
where
    Tag: tag::Tag1<T>,
{
    /// Create a new `SetUnion` from a `Set`.
    pub fn new(val: Tag::Bind) -> Self {
        Self(val)
    }

    /// Create a new `SetUnion` from an `Into<Set>`.
    pub fn new_from(val: impl Into<Tag::Bind>) -> Self {
        Self::new(val.into())
    }
}

impl<TagSelf, TagOther, T> Merge<SetUnion<TagOther, T>> for SetUnion<TagSelf, T>
where
    TagSelf: tag::Tag1<T>,
    TagOther: tag::Tag1<T>,
    TagSelf::Bind: Collection<T, ()> + Extend<T>,
    TagOther::Bind: IntoIterator<Item = T>,
{
    fn merge(&mut self, other: SetUnion<TagOther, T>) -> bool {
        let old_len = self.0.len();
        self.0.extend(other.0);
        self.0.len() > old_len
    }
}

impl<TagSelf, TagOther, T> ConvertFrom<SetUnion<TagOther, T>> for SetUnion<TagSelf, T>
where
    TagSelf: tag::Tag1<T>,
    TagOther: tag::Tag1<T>,
    TagSelf::Bind: FromIterator<T>,
    TagOther::Bind: Collection<T, ()>,
{
    fn from(other: SetUnion<TagOther, T>) -> Self {
        Self(
            other
                .0
                .into_entries()
                .map(|(t_other, ())| t_other)
                .collect(),
        )
    }
}

impl<Tag, T> Default for SetUnion<Tag, T>
where
    Tag: tag::Tag1<T>,
    Tag::Bind: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<TagSelf, TagOther, T> Compare<SetUnion<TagOther, T>> for SetUnion<TagSelf, T>
where
    TagSelf: tag::Tag1<T>,
    TagOther: tag::Tag1<T>,
    TagSelf::Bind: Collection<T, ()>,
    TagOther::Bind: Collection<T, ()>,
{
    fn compare(&self, other: &SetUnion<TagOther, T>) -> Option<Ordering> {
        match self.0.len().cmp(&other.0.len()) {
            Ordering::Greater => {
                if other.0.keys().all(|key| self.0.get(key).is_some()) {
                    Some(Ordering::Greater)
                } else {
                    None
                }
            }
            Ordering::Equal => {
                if self.0.keys().all(|key| other.0.get(key).is_some()) {
                    Some(Ordering::Equal)
                } else {
                    None
                }
            }
            Ordering::Less => {
                if self.0.keys().all(|key| other.0.get(key).is_some()) {
                    Some(Ordering::Less)
                } else {
                    None
                }
            }
        }
    }
}

/// [`std::collections::HashSet`]-backed [`SetUnion`] lattice.
pub type SetUnionHashSet<T> = SetUnion<tag::HASH_SET, T>;

/// [`std::collections::BTreeSet`]-backed [`SetUnion`] lattice.
pub type SetUnionBTreeSet<T> = SetUnion<tag::BTREE_SET, T>;

/// [`Vec`]-backed [`SetUnion`] lattice.
pub type SetUnionVec<T> = SetUnion<tag::VEC, T>;

/// Array-backed [`SetUnion`] lattice.
pub type SetUnionArray<T, const N: usize> = SetUnion<tag::ARRAY<N>, T>;

/// [`crate::collections::MaskedArray`]-backed [`SetUnion`] lattice.
pub type SetUnionMaskedArray<T, const N: usize> = SetUnion<tag::MASKED_ARRAY<N>, T>;

/// [`crate::collections::Single`]-backed [`SetUnion`] lattice.
pub type SetUnionSingle<T> = SetUnion<tag::SINGLE, T>;

/// [`Option`]-backed [`SetUnion`] lattice.
pub type SetUnionOption<T> = SetUnion<tag::OPTION, T>;

#[cfg(test)]
mod test {
    use super::*;

    use crate::collections::Single;

    #[test]
    fn test_set_union() {
        let mut my_set_a = SetUnion::<tag::HASH_SET, &str>(Default::default());
        let my_set_b = SetUnion::<tag::BTREE_SET, &str>(Default::default());
        let my_set_c = SetUnion::<tag::SINGLE, _>(Single("hello world"));

        assert_eq!(Some(Ordering::Equal), my_set_a.compare(&my_set_a));
        assert_eq!(Some(Ordering::Equal), my_set_a.compare(&my_set_b));
        assert_eq!(Some(Ordering::Less), my_set_a.compare(&my_set_c));
        assert_eq!(Some(Ordering::Equal), my_set_b.compare(&my_set_a));
        assert_eq!(Some(Ordering::Equal), my_set_b.compare(&my_set_b));
        assert_eq!(Some(Ordering::Less), my_set_b.compare(&my_set_c));
        assert_eq!(Some(Ordering::Greater), my_set_c.compare(&my_set_a));
        assert_eq!(Some(Ordering::Greater), my_set_c.compare(&my_set_b));
        assert_eq!(Some(Ordering::Equal), my_set_c.compare(&my_set_c));

        my_set_a.merge(my_set_b);
        my_set_a.merge(my_set_c);
    }

    #[test]
    fn test_singleton_example() {
        let mut my_hash_set = SetUnionHashSet::<&str>::default();
        let my_delta_set = SetUnionSingle::new_from("hello world");
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
