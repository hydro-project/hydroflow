use std::cmp::Ordering;
use std::iter::FromIterator;

use crate::lang::collections::Collection;
use crate::lang::lattice::{Compare, Convert, Lattice, LatticeRepr, Merge};
use crate::lang::tag;

pub struct MapUnion<K, L: Lattice> {
    _phantom: std::marker::PhantomData<(K, L)>,
}
impl<K, L: Lattice> Lattice for MapUnion<K, L> {}

pub trait MapTag<T, U>: tag::Tag2<T, U> {}
impl<T, U> MapTag<T, U> for tag::HASH_MAP {}
impl<T, U> MapTag<T, U> for tag::BTREE_MAP {}
impl<T, U> MapTag<T, U> for tag::VEC {}
impl<T, U> MapTag<T, U> for tag::SINGLE {}
impl<T, U> MapTag<T, U> for tag::OPTION {}
impl<T, U, const N: usize> MapTag<T, U> for tag::ARRAY<N> {}
impl<T, U, const N: usize> MapTag<T, U> for tag::MASKED_ARRAY<N> {}

pub struct MapUnionRepr<Tag: MapTag<K, B::Repr>, K, B: LatticeRepr> {
    _phantom: std::marker::PhantomData<(Tag, K, B)>,
}

impl<Tag: MapTag<K, B::Repr>, K, B: LatticeRepr> LatticeRepr for MapUnionRepr<Tag, K, B>
where
    Tag::Bind: Clone,
{
    type Lattice = MapUnion<K, B::Lattice>;
    type Repr = Tag::Bind;
}

impl<
        K: 'static,
        SelfTag,
        DeltaTag,
        SelfLr: LatticeRepr<Lattice = L>,
        DeltaLr: LatticeRepr<Lattice = L>,
        L: Lattice,
    > Merge<MapUnionRepr<DeltaTag, K, DeltaLr>> for MapUnionRepr<SelfTag, K, SelfLr>
where
    SelfTag: MapTag<K, SelfLr::Repr>,
    DeltaTag: MapTag<K, DeltaLr::Repr>,
    MapUnionRepr<SelfTag, K, SelfLr>: LatticeRepr<Lattice = MapUnion<K, L>>,
    MapUnionRepr<DeltaTag, K, DeltaLr>: LatticeRepr<Lattice = MapUnion<K, L>>,
    <MapUnionRepr<SelfTag, K, SelfLr> as LatticeRepr>::Repr:
        Extend<(K, SelfLr::Repr)> + Collection<K, SelfLr::Repr>,
    <MapUnionRepr<DeltaTag, K, DeltaLr> as LatticeRepr>::Repr:
        IntoIterator<Item = (K, DeltaLr::Repr)>,
    SelfLr: Merge<DeltaLr>,
    DeltaLr: Convert<SelfLr>,
{
    fn merge(
        this: &mut <MapUnionRepr<SelfTag, K, SelfLr> as LatticeRepr>::Repr,
        delta: <MapUnionRepr<DeltaTag, K, DeltaLr> as LatticeRepr>::Repr,
    ) -> bool {
        let mut changed = false;
        let iter: Vec<(K, SelfLr::Repr)> = delta
            .into_iter()
            .filter_map(|(k, v)| {
                match this.get_mut(&k) {
                    // Key collision, merge into THIS.
                    Some(target_val) => {
                        changed |= <SelfLr as Merge<DeltaLr>>::merge(target_val, v);
                        None
                    }
                    // New value, convert for extending.
                    None => {
                        changed = true;
                        let val: SelfLr::Repr = <DeltaLr as Convert<SelfLr>>::convert(v);
                        Some((k, val))
                    }
                }
            })
            .collect();
        this.extend(iter);
        changed
    }
}

impl<K, SelfInner: LatticeRepr, SelfTag, TargetInner: LatticeRepr, TargetTag>
    Convert<MapUnionRepr<TargetTag, K, TargetInner>> for MapUnionRepr<SelfTag, K, SelfInner>
where
    SelfTag: MapTag<K, SelfInner::Repr>,
    TargetTag: MapTag<K, TargetInner::Repr>,
    SelfInner: LatticeRepr<Lattice = TargetInner::Lattice>,
    SelfInner: Convert<TargetInner>,
    MapUnionRepr<SelfTag, K, SelfInner>: LatticeRepr<Lattice = MapUnion<K, SelfInner::Lattice>>,
    MapUnionRepr<TargetTag, K, TargetInner>:
        LatticeRepr<Lattice = MapUnion<K, TargetInner::Lattice>>,
    <MapUnionRepr<SelfTag, K, SelfInner> as LatticeRepr>::Repr:
        IntoIterator<Item = (K, SelfInner::Repr)>,
    <MapUnionRepr<TargetTag, K, TargetInner> as LatticeRepr>::Repr:
        FromIterator<(K, TargetInner::Repr)>,
{
    fn convert(
        this: <MapUnionRepr<SelfTag, K, SelfInner> as LatticeRepr>::Repr,
    ) -> <MapUnionRepr<TargetTag, K, TargetInner> as LatticeRepr>::Repr {
        this.into_iter()
            .map(|(k, val)| (k, <SelfInner as Convert<TargetInner>>::convert(val)))
            .collect()
    }
}

impl<
        K: 'static,
        SelfTag,
        DeltaTag,
        SelfLr: LatticeRepr<Lattice = L>,
        DeltaLr: LatticeRepr<Lattice = L>,
        L: Lattice,
    > Compare<MapUnionRepr<DeltaTag, K, DeltaLr>> for MapUnionRepr<SelfTag, K, SelfLr>
where
    SelfTag: MapTag<K, SelfLr::Repr>,
    DeltaTag: MapTag<K, DeltaLr::Repr>,
    MapUnionRepr<SelfTag, K, SelfLr>: LatticeRepr<Lattice = MapUnion<K, L>>,
    MapUnionRepr<DeltaTag, K, DeltaLr>: LatticeRepr<Lattice = MapUnion<K, L>>,
    <MapUnionRepr<SelfTag, K, SelfLr> as LatticeRepr>::Repr: Collection<K, SelfLr::Repr>,
    <MapUnionRepr<DeltaTag, K, DeltaLr> as LatticeRepr>::Repr: Collection<K, DeltaLr::Repr>,
    SelfLr: Compare<DeltaLr>,
{
    fn compare(
        this: &<MapUnionRepr<SelfTag, K, SelfLr> as LatticeRepr>::Repr,
        other: &<MapUnionRepr<DeltaTag, K, DeltaLr> as LatticeRepr>::Repr,
    ) -> Option<Ordering> {
        match this.len().cmp(&other.len()) {
            Ordering::Greater => {
                for (key, this_value) in this.entries() {
                    if let Some(other_value) = other.get(key) {
                        if let Some(Ordering::Less) = SelfLr::compare(this_value, other_value) {
                            return None;
                        }
                    }
                }
                Some(Ordering::Greater)
            }
            Ordering::Equal => {
                let mut current_ordering = Ordering::Equal;
                for (key, this_value) in this.entries() {
                    match other.get(key) {
                        Some(other_value) => {
                            match SelfLr::compare(this_value, other_value) {
                                // current_ordering unchanged
                                Some(Ordering::Equal) => {}
                                // If we get a strict inequality, check if that conflicts with the current_ordering.
                                // Then update the current_ordering.
                                Some(inequal) => {
                                    if inequal.reverse() == current_ordering {
                                        // Conflict.
                                        return None;
                                    }
                                    current_ordering = inequal;
                                }
                                None => return None,
                            }
                        }
                        None => {
                            if Ordering::Less == current_ordering {
                                return None;
                            }
                        }
                    }
                }
                Some(current_ordering)
            }
            Ordering::Less => {
                // this.len() < other.len()
                for (key, other_value) in other.entries() {
                    if let Some(this_value) = this.get(key) {
                        if let Some(Ordering::Greater) = SelfLr::compare(this_value, other_value) {
                            return None;
                        }
                    }
                }
                Some(Ordering::Less)
            }
        }
    }
}

// impl<Tag: MapTag<K, B::Repr>, K, B: LatticeRepr> Bottom for MapUnionRepr<Tag, K, B>
// where
//     Tag::Bind: Clone,
//     Self::Repr: Collection<K, B::Repr>,
// {
//     fn is_bottom(this: &Self::Repr) -> bool {
//         this.is_empty()
//     }
// }

fn __assert_merges() {
    use static_assertions::{assert_impl_all, assert_not_impl_any};

    use super::set_union::SetUnionRepr;

    type HashMapHashSet = MapUnionRepr<tag::HASH_MAP, String, SetUnionRepr<tag::HASH_SET, u32>>;
    type HashMapArraySet = MapUnionRepr<tag::HASH_MAP, String, SetUnionRepr<tag::ARRAY<8>, u32>>;
    type OptionMapArraySet = MapUnionRepr<tag::OPTION, String, SetUnionRepr<tag::HASH_SET, u32>>;

    assert_impl_all!(HashMapHashSet: Merge<HashMapHashSet>);
    assert_impl_all!(HashMapHashSet: Merge<HashMapArraySet>);

    assert_not_impl_any!(HashMapArraySet: Merge<HashMapHashSet>);
    assert_not_impl_any!(HashMapArraySet: Merge<HashMapArraySet>);

    assert_not_impl_any!(OptionMapArraySet: Merge<HashMapHashSet>);
    assert_not_impl_any!(OptionMapArraySet: Merge<HashMapArraySet>);
}
