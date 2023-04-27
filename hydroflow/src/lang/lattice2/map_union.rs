//! Map-union compound lattice.
//!
//! Each key corresponds to a lattice value instance. Merging map-union lattices is done by
//! unioning the keys and merging the values of intersecting keys.

use crate::lang::{collections::Collection, tag};

use super::{ConvertFrom, Merge};

/// A map-union lattice.
///
/// `Tag` specifies what datastructure to use, allowing us to deal with different datastructures generically.
#[repr(transparent)]
pub struct MapUnion<Tag, K, Val>(pub Tag::Bind)
where
    Tag: tag::Tag2<K, Val>;
impl<Tag, K, Val> MapUnion<Tag, K, Val>
where
    Tag: tag::Tag2<K, Val>,
{
    /// Create a new `MapUnion` from a map.
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

/// [`std::collections::HashMap`]-backed [`MapUnion`] lattice.
pub type MapUnionHashMap<K, Val> = MapUnion<tag::HASH_MAP, K, Val>;

/// [`std::collections::BTreeMap`]-backed [`MapUnion`] lattice.
pub type MapUnionBTreeMap<K, Val> = MapUnion<tag::BTREE_MAP, K, Val>;

/// [`Vec`]-backed [`MapUnion`] lattice.
pub type MapUnionVec<K, Val> = MapUnion<tag::VEC, K, Val>;

/// Array-backed [`MapUnion`] lattice.
pub type MapUnionArray<K, Val, const N: usize> = MapUnion<tag::ARRAY<N>, K, Val>;

/// [`crate::lang::collections::MaskedArray`]-backed [`MapUnion`] lattice.
pub type MapUnionMaskedArray<K, Val, const N: usize> = MapUnion<tag::MASKED_ARRAY<N>, K, Val>;

/// [`crate::lang::collections::Single`]-backed [`MapUnion`] lattice.
pub type MapUnionSingle<K, Val> = MapUnion<tag::SINGLE, K, Val>;

/// [`Option`]-backed [`MapUnion`] lattice.
pub type MapUnionOption<K, Val> = MapUnion<tag::OPTION, K, Val>;

#[cfg(test)]
mod test {
    use super::*;

    use crate::lang::collections::Single;
    use crate::lang::lattice2::set_union::{SetUnionHashSet, SetUnionSingle};

    #[test]
    fn test_map_union() {
        let mut my_map_a = <MapUnionHashMap<&str, SetUnionHashSet<u64>>>::default();
        let my_map_b = <MapUnionSingle<&str, SetUnionSingle<u64>>>::new(Single((
            "hello",
            SetUnionSingle::new(Single(100)),
        )));
        let my_map_c = MapUnionSingle::new(("hello", SetUnionHashSet::new([100, 200])));
        my_map_a.merge(my_map_b);
        my_map_a.merge(my_map_c);
    }
}
