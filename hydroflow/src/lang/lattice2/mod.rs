use std::cmp::Ordering;

use super::collections::{Collection, Single};
use super::tag;

pub trait Merge<Other> {
    fn merge(&mut self, other: Other) -> bool;
}

/// Same as `From` but ONLY FOR LATTICES.
/// Do not convert non-lattice (AKA scalar) types if you implement this trait.
pub trait ConvertFrom<Other> {
    fn from(other: Other) -> Self;
}

pub trait Compare<Other> {
    fn compare(&self, other: &Other) -> Option<Ordering>;
}

#[repr(transparent)]
#[derive(Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Max<T>(T);
impl<T> Max<T> {
    pub fn new(val: impl Into<T>) -> Self {
        Self(val.into())
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

#[repr(transparent)]
#[derive(Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Min<T>(pub T);
impl<T> Min<T> {
    pub fn new(val: impl Into<T>) -> Self {
        Self(val.into())
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

// #[repr(transparent)]
// pub struct SetUnion<T>(pub HashSet<T>);
// impl<T> Merge<SetUnion<T>> for SetUnion<T>
// where
//     T: Eq + Hash,
// {
//     fn merge(&mut self, other: SetUnion<T>) -> bool {
//         let old_len = self.0.len();
//         self.0.extend(other.0);
//         self.0.len() > old_len
//     }
// }

#[repr(transparent)]
pub struct SetUnion<Tag, T>(Tag::Bind)
where
    Tag: tag::Tag1<T>;
impl<Tag, T> SetUnion<Tag, T>
where
    Tag: tag::Tag1<T>,
{
    pub fn new(val: impl Into<Tag::Bind>) -> Self {
        Self(val.into())
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
                if self.0.keys().all(|key| other.0.get(key).is_some()) {
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
                if other.0.keys().all(|key| self.0.get(key).is_some()) {
                    Some(Ordering::Less)
                } else {
                    None
                }
            }
        }
    }
}
pub type SetUnionHashSet<T> = SetUnion<tag::HASH_SET, T>;
pub type SetUnionBTreeSet<T> = SetUnion<tag::BTREE_SET, T>;
pub type SetUnionVec<T> = SetUnion<tag::VEC, T>;
pub type SetUnionArray<T, const N: usize> = SetUnion<tag::ARRAY<N>, T>;
pub type SetUnionMaskedArray<T, const N: usize> = SetUnion<tag::MASKED_ARRAY<N>, T>;
pub type SetUnionSingle<T> = SetUnion<tag::SINGLE, T>;
pub type SetUnionOption<T> = SetUnion<tag::OPTION, T>;

#[test]
fn test_set_union() {
    let mut my_set_a = SetUnion::<tag::HASH_SET, &str>(Default::default());
    let my_set_b = SetUnion::<tag::BTREE_SET, &str>(Default::default());
    let my_set_c = SetUnion::<tag::SINGLE, _>(Single("hello world"));
    my_set_a.merge(my_set_b);
    my_set_a.merge(my_set_c);
}

#[repr(transparent)]
pub struct MapUnion<Tag, K, Val>(pub Tag::Bind)
where
    Tag: tag::Tag2<K, Val>;
impl<Tag, K, Val> MapUnion<Tag, K, Val>
where
    Tag: tag::Tag2<K, Val>,
{
    pub fn new(val: impl Into<Tag::Bind>) -> Self {
        Self(val.into())
    }
}
impl<TagSelf, TagOther, K, ValSelf, ValOther> Merge<MapUnion<TagOther, K, ValOther>>
    for MapUnion<TagSelf, K, ValSelf>
where
    TagSelf: tag::Tag2<K, ValSelf>,
    TagOther: tag::Tag2<K, ValOther>,
    TagSelf::Bind: Collection<K, ValSelf> + Extend<(K, ValSelf)>,
    TagOther::Bind: IntoIterator<Item = (K, ValOther)>,
    ValSelf: Merge<ValOther> + ConvertFrom<ValOther>,
{
    fn merge(&mut self, other: MapUnion<TagOther, K, ValOther>) -> bool {
        let mut changed = false;
        let iter: Vec<_> = other
            .0
            .into_iter()
            .filter_map(|(k_other, val_other)| {
                match self.0.get_mut(&k_other) {
                    // Key collision, merge into THIS.
                    Some(val_self) => {
                        changed |= val_self.merge(val_other);
                        None
                    }
                    // New value, convert for extending.
                    None => Some((k_other, ValSelf::from(val_other))),
                }
            })
            .collect();
        self.0.extend(iter);
        changed
    }
}
impl<TagSelf, TagOther, K, ValSelf, ValOther> ConvertFrom<MapUnion<TagOther, K, ValOther>>
    for MapUnion<TagSelf, K, ValSelf>
where
    TagSelf: tag::Tag2<K, ValSelf>,
    TagOther: tag::Tag2<K, ValOther>,
    TagSelf::Bind: FromIterator<(K, ValSelf)>,
    TagOther::Bind: Collection<K, ValOther>,
    ValSelf: ConvertFrom<ValOther>,
{
    fn from(other: MapUnion<TagOther, K, ValOther>) -> Self {
        Self(
            other
                .0
                .into_entries()
                .map(|(k_other, val_other)| (k_other, ConvertFrom::from(val_other)))
                .collect(),
        )
    }
}
impl<Tag, K, Val> Default for MapUnion<Tag, K, Val>
where
    Tag: tag::Tag2<K, Val>,
    Tag::Bind: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}
pub type MapUnionHashMap<K, Val> = MapUnion<tag::HASH_MAP, K, Val>;
pub type MapUnionBTreeMap<K, Val> = MapUnion<tag::BTREE_MAP, K, Val>;
pub type MapUnionVec<K, Val> = MapUnion<tag::VEC, K, Val>;
pub type MapUnionArray<K, Val, const N: usize> = MapUnion<tag::ARRAY<N>, K, Val>;
pub type MapUnionMaskedArray<K, Val, const N: usize> = MapUnion<tag::MASKED_ARRAY<N>, K, Val>;
pub type MapUnionSingle<K, Val> = MapUnion<tag::SINGLE, K, Val>;
pub type MapUnionOption<K, Val> = MapUnion<tag::OPTION, K, Val>;

#[test]
fn test_map_union() {
    // impl From<SetUnion<tag::SINGLE, u64>> for SetUnion<tag::HASH_SET, u64> {
    //     fn from(value: SetUnion<tag::SINGLE, u64>) -> Self {
    //         SetUnion(std::iter::once(value.0 .0).collect())
    //     }
    // }

    let mut my_map_a = <MapUnionHashMap<&str, SetUnionHashSet<u64>>>::default();
    let my_map_b = <MapUnionSingle<&str, SetUnionSingle<u64>>>::new(Single((
        "hello",
        SetUnionSingle::new(Single(100)),
    )));
    let my_map_c = MapUnionSingle::new(("hello", SetUnionHashSet::new([100, 200])));
    my_map_a.merge(my_map_b);
    my_map_a.merge(my_map_c);
}
