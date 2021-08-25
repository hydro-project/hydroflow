use std::iter::FromIterator;
use std::cmp::Ordering;

use super::*;

use crate::collections::*;
use crate::tag;

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

impl<Tag: MapTag<K, B::Repr>, K: 'static, B: LatticeRepr> LatticeRepr for MapUnionRepr<Tag, K, B>
where
    Tag::Bind: Clone,
{
    type Lattice = MapUnion<K, B::Lattice>;
    type Repr = Tag::Bind;
}

impl<K: 'static, SelfTag, DeltaTag, SelfLr: LatticeRepr<Lattice = L>, DeltaLr: LatticeRepr<Lattice = L>, L: Lattice> Merge<MapUnionRepr<DeltaTag, K, DeltaLr>> for MapUnionRepr<SelfTag, K, SelfLr>
where
    SelfTag:  MapTag<K, SelfLr::Repr>,
    DeltaTag: MapTag<K, DeltaLr::Repr>,
    MapUnionRepr<SelfTag,  K, SelfLr>:  LatticeRepr<Lattice = MapUnion<K, L>>,
    MapUnionRepr<DeltaTag, K, DeltaLr>: LatticeRepr<Lattice = MapUnion<K, L>>,
    <MapUnionRepr<SelfTag,  K, SelfLr>  as LatticeRepr>::Repr: Extend<(K, SelfLr::Repr)> + Collection<K, SelfLr::Repr>,
    <MapUnionRepr<DeltaTag, K, DeltaLr> as LatticeRepr>::Repr: IntoIterator<Item = (K, DeltaLr::Repr)>,
    SelfLr:  Merge<DeltaLr>,
    DeltaLr: Convert<SelfLr>,
{
    fn merge(this: &mut <MapUnionRepr<SelfTag, K, SelfLr> as LatticeRepr>::Repr, delta: <MapUnionRepr<DeltaTag, K, DeltaLr> as LatticeRepr>::Repr) -> bool {
        let mut changed = false;
        let iter: Vec<(K, SelfLr::Repr)> = delta.into_iter()
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

impl<K, SelfInner: LatticeRepr, SelfTag, TargetInner: LatticeRepr, TargetTag> Convert<MapUnionRepr<TargetTag, K, TargetInner>> for MapUnionRepr<SelfTag, K, SelfInner>
where
    SelfTag: MapTag<K, SelfInner::Repr>,
    TargetTag: MapTag<K, TargetInner::Repr>,
    SelfInner: LatticeRepr<Lattice = TargetInner::Lattice>,
    SelfInner: Convert<TargetInner>,
    MapUnionRepr<SelfTag,   K, SelfInner>:   LatticeRepr<Lattice = MapUnion<K, SelfInner::Lattice>>,
    MapUnionRepr<TargetTag, K, TargetInner>: LatticeRepr<Lattice = MapUnion<K, TargetInner::Lattice>>,
    <MapUnionRepr<SelfTag,   K, SelfInner>   as LatticeRepr>::Repr: IntoIterator<Item = (K, SelfInner::Repr)>,
    <MapUnionRepr<TargetTag, K, TargetInner> as LatticeRepr>::Repr: FromIterator<(K, TargetInner::Repr)>,
{
    fn convert(this: <MapUnionRepr<SelfTag, K, SelfInner> as LatticeRepr>::Repr) -> <MapUnionRepr<TargetTag, K, TargetInner> as LatticeRepr>::Repr {
        this.into_iter()
            .map(|(k, val)| (k, <SelfInner as Convert<TargetInner>>::convert(val)))
            .collect()
    }
}

impl<K: 'static, SelfTag, DeltaTag, SelfLr: LatticeRepr<Lattice = L>, DeltaLr: LatticeRepr<Lattice = L>, L: Lattice> Compare<MapUnionRepr<DeltaTag, K, DeltaLr>> for MapUnionRepr<SelfTag, K, SelfLr>
where
    SelfTag:  MapTag<K, SelfLr::Repr>,
    DeltaTag: MapTag<K, DeltaLr::Repr>,
    MapUnionRepr<SelfTag,  K, SelfLr>:  LatticeRepr<Lattice = MapUnion<K, L>>,
    MapUnionRepr<DeltaTag, K, DeltaLr>: LatticeRepr<Lattice = MapUnion<K, L>>,
    <MapUnionRepr<SelfTag,  K, SelfLr>  as LatticeRepr>::Repr: Collection<K, SelfLr::Repr>,
    <MapUnionRepr<DeltaTag, K, DeltaLr> as LatticeRepr>::Repr: Collection<K, DeltaLr::Repr>,
    SelfLr: Compare<DeltaLr>,
{
    fn compare(this: &<MapUnionRepr<SelfTag, K, SelfLr> as LatticeRepr>::Repr, other: &<MapUnionRepr<DeltaTag, K, DeltaLr> as LatticeRepr>::Repr) -> Option<Ordering> {
        if this.len() > other.len() {
            for (key, this_value) in this.entries() {
                if let Some(other_value) = other.get(key) {
                    if let Some(Ordering::Less) = SelfLr::compare(this_value, other_value) {
                        return None;
                    }
                }
            }
            Some(Ordering::Greater)
        }
        else if this.len() == other.len() {
            let mut current_ordering = Ordering::Equal;
            for (key, this_value) in this.entries() {
                match other.get(key) {
                    Some(other_value) => {
                        match SelfLr::compare(this_value, other_value) {
                            // current_ordering unchanged
                            Some(Ordering::Equal) => {},
                            // If we get a strict inequality, check if that conflicts with the current_ordering.
                            // Then update the current_ordering.
                            Some(inequal) => {
                                if inequal.reverse() == current_ordering {
                                    // Conflict.
                                    return None;
                                }
                                current_ordering = inequal;
                            }
                            None => return None
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
        else { // this.len() < other.len()
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

// impl<Tag: MapTag<K, B::Repr>, K, B: LatticeRepr> Bottom for MapUnionRepr<Tag, K, B>
// where
//     Tag::Bind: Clone,
//     Self::Repr: Collection<K, B::Repr>,
// {
//     fn is_bottom(this: &Self::Repr) -> bool {
//         this.is_empty()
//     }
// }

mod fns {
    use crate::hide::{Hide, Qualifier};
    use crate::lattice::set_union::{SetTag, SetUnion, SetUnionRepr};

    use super::*;

    // impl<Y: Qualifier, K: Clone, V: Clone, Tag, Lr: LatticeRepr> Hide<Y, MapUnionRepr<Tag, K, Lr>>
    // where
    // {
    //     pub fn map<TargetTag>(self) -> Hide<Y, SetUnionRepr<TargetTag, (K, V)>>
    //     where
    //     {
    //     }
    // }

    impl<Y: Qualifier, K: Clone, InnerK: Clone, Tag, InnerTag, InnermostLr> Hide<Y, MapUnionRepr<Tag, K, MapUnionRepr<InnerTag, InnerK, InnermostLr>>>
    where
        InnermostLr: LatticeRepr,
        InnerTag: MapTag<InnerK, InnermostLr::Repr>,
        MapUnionRepr<InnerTag, InnerK, InnermostLr>: LatticeRepr,
        <MapUnionRepr<InnerTag, InnerK, InnermostLr> as LatticeRepr>::Repr: IntoIterator<Item = (InnerK, InnermostLr::Repr)>,

        Tag: MapTag<K, <MapUnionRepr<InnerTag, InnerK, InnermostLr> as LatticeRepr>::Repr>,
        MapUnionRepr<Tag, K, MapUnionRepr<InnerTag, InnerK, InnermostLr>>: LatticeRepr,
        <MapUnionRepr<Tag, K, MapUnionRepr<InnerTag, InnerK, InnermostLr>> as LatticeRepr>::Repr: IntoIterator<Item = (K, <MapUnionRepr<InnerTag, InnerK, InnermostLr> as LatticeRepr>::Repr)>,
    {
        pub fn transpose<TargetTag, TargetInnerTag>(self) -> Hide<Y, MapUnionRepr<TargetTag, InnerK, MapUnionRepr<TargetInnerTag, K, InnermostLr>>>
        where
            TargetInnerTag: MapTag<K, InnermostLr::Repr>,
            MapUnionRepr<TargetInnerTag, K, InnermostLr>: LatticeRepr,

            TargetTag: MapTag<InnerK, <MapUnionRepr<TargetInnerTag, K, InnermostLr> as LatticeRepr>::Repr>,
            MapUnionRepr<TargetTag, InnerK, MapUnionRepr<TargetInnerTag, K, InnermostLr>>: LatticeRepr,
            MapUnionRepr<TargetTag, InnerK, MapUnionRepr<TargetInnerTag, K, InnermostLr>>: Merge<MapUnionRepr<tag::SINGLE, InnerK, MapUnionRepr<tag::SINGLE, K, InnermostLr>>>,
            <MapUnionRepr<TargetTag, InnerK, MapUnionRepr<TargetInnerTag, K, InnermostLr>> as LatticeRepr>::Repr: Default,
        {
            let mut out = <<MapUnionRepr<TargetTag, InnerK, MapUnionRepr<TargetInnerTag, K, InnermostLr>> as LatticeRepr>::Repr as Default>::default();

            for (outer_k, inner_map) in self.into_reveal().into_iter() {
                for (inner_k, value) in inner_map.into_iter() {
                    <MapUnionRepr<TargetTag, InnerK, MapUnionRepr<TargetInnerTag, K, InnermostLr>> as Merge<MapUnionRepr<tag::SINGLE, InnerK, MapUnionRepr<tag::SINGLE, K, InnermostLr>>>>
                        ::merge(&mut out, Single((inner_k, Single((outer_k.clone(), value)))));
                }
            }

            Hide::new(out)
        }
    }

    impl<Y: Qualifier, K: Clone, Tag, InnerLr: LatticeRepr> Hide<Y, MapUnionRepr<Tag, K, InnerLr>>
    where
        Tag: MapTag<K, InnerLr::Repr>,
        MapUnionRepr<Tag, K, InnerLr>: LatticeRepr,
        <MapUnionRepr<Tag, K, InnerLr> as LatticeRepr>::Repr: IntoIterator<Item = (K, InnerLr::Repr)>,
    {
        pub fn fold_values<TargetLr>(self) -> Hide<Y, TargetLr>
        where
            TargetLr: LatticeRepr + Merge<InnerLr>,
            TargetLr::Repr: Default,
        {
            let mut out = <TargetLr::Repr as Default>::default();
            for (_key, val) in self.into_reveal().into_iter() {
                <TargetLr as Merge<InnerLr>>::merge(&mut out, val);
            }
            Hide::new(out)
        }
    }

    impl<Y: Qualifier, K: Clone, V: Clone, Tag, SetUnionLr> Hide<Y, MapUnionRepr<Tag, K, SetUnionLr>>
    where
        SetUnionLr: LatticeRepr<Lattice = SetUnion<V>>,
        <SetUnionLr as LatticeRepr>::Repr: IntoIterator<Item = V>,
        Tag: MapTag<K, SetUnionLr::Repr>,
        MapUnionRepr<Tag, K, SetUnionLr>: LatticeRepr,
        <MapUnionRepr<Tag, K, SetUnionLr> as LatticeRepr>::Repr: IntoIterator<Item = (K, SetUnionLr::Repr)>,
    {
        pub fn flatten_keyed<TargetTag>(self) -> Hide<Y, SetUnionRepr<TargetTag, (K, V)>>
        where
            TargetTag: SetTag<(K, V)>,
            SetUnionRepr<TargetTag, (K, V)>: LatticeRepr + Merge<SetUnionRepr<tag::SINGLE, (K, V)>>,
            <SetUnionRepr<TargetTag, (K, V)> as LatticeRepr>::Repr: Default,
        {
            let mut out = <<SetUnionRepr<TargetTag, (K, V)> as LatticeRepr>::Repr as Default>::default();
            for (key, vals) in self.into_reveal().into_iter() {
                for val in vals.into_iter() {
                    <SetUnionRepr<TargetTag, (K, V)> as Merge<SetUnionRepr<tag::SINGLE, (K, V)>>>::merge(&mut out, Single((key.clone(), val)));
                }
            }
            Hide::new(out)
        }
    }
}

fn __assert_merges() {
    use static_assertions::{assert_impl_all, assert_not_impl_any};

    use super::set_union::{SetUnionRepr};

    type HashMapHashSet    = MapUnionRepr<tag::HASH_MAP, String, SetUnionRepr<tag::HASH_SET, u32>>;
    type HashMapArraySet   = MapUnionRepr<tag::HASH_MAP, String, SetUnionRepr<tag::ARRAY<8>, u32>>;
    type OptionMapArraySet = MapUnionRepr<tag::OPTION,   String, SetUnionRepr<tag::HASH_SET, u32>>;

    assert_impl_all!(HashMapHashSet: Merge<HashMapHashSet>);
    assert_impl_all!(HashMapHashSet: Merge<HashMapArraySet>);

    assert_not_impl_any!(HashMapArraySet: Merge<HashMapHashSet>);
    assert_not_impl_any!(HashMapArraySet: Merge<HashMapArraySet>);

    assert_not_impl_any!(OptionMapArraySet: Merge<HashMapHashSet>);
    assert_not_impl_any!(OptionMapArraySet: Merge<HashMapArraySet>);
}
