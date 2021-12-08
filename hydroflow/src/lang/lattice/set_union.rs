use std::cmp::Ordering;
use std::iter::FromIterator;

use super::{Compare, Convert, Debottom, Lattice, LatticeRepr, Merge};

use crate::lang::collections::Collection;
use crate::lang::tag;

pub struct SetUnion<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T> Lattice for SetUnion<T> {}

pub trait SetTag<T>: tag::Tag1<T> {}
impl<T> SetTag<T> for tag::HASH_SET {}
impl<T> SetTag<T> for tag::BTREE_SET {}
impl<T> SetTag<T> for tag::VEC {}
impl<T> SetTag<T> for tag::SINGLE {}
impl<T> SetTag<T> for tag::OPTION {}
impl<T, const N: usize> SetTag<T> for tag::ARRAY<N> {}
impl<T, const N: usize> SetTag<T> for tag::MASKED_ARRAY<N> {}

pub struct SetUnionRepr<Tag: SetTag<T>, T> {
    _phantom: std::marker::PhantomData<(Tag, T)>,
}

impl<Tag: SetTag<T>, T> LatticeRepr for SetUnionRepr<Tag, T>
where
    Tag::Bind: Clone,
{
    type Lattice = SetUnion<T>;
    type Repr = Tag::Bind;
}

impl<T, SelfTag: SetTag<T>, DeltaTag: SetTag<T>> Merge<SetUnionRepr<DeltaTag, T>>
    for SetUnionRepr<SelfTag, T>
where
    SetUnionRepr<SelfTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    SetUnionRepr<DeltaTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    <SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr: Collection<T, ()> + Extend<T>,
    <SetUnionRepr<DeltaTag, T> as LatticeRepr>::Repr: IntoIterator<Item = T>,
{
    fn merge(
        this: &mut <SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr,
        delta: <SetUnionRepr<DeltaTag, T> as LatticeRepr>::Repr,
    ) -> bool {
        let old_len = this.len();
        this.extend(delta);
        this.len() > old_len
    }
}

impl<T, SelfTag: SetTag<T>, TargetTag: SetTag<T>> Convert<SetUnionRepr<TargetTag, T>>
    for SetUnionRepr<SelfTag, T>
where
    SetUnionRepr<SelfTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    SetUnionRepr<TargetTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    <SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr: IntoIterator<Item = T>,
    <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr: FromIterator<T>,
{
    fn convert(
        this: <SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr,
    ) -> <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr {
        this.into_iter().collect()
    }
}

impl<T: 'static, SelfTag: SetTag<T>, TargetTag: SetTag<T>> Compare<SetUnionRepr<TargetTag, T>>
    for SetUnionRepr<SelfTag, T>
where
    SetUnionRepr<SelfTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    SetUnionRepr<TargetTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    <SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr: Collection<T, ()>,
    <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr: Collection<T, ()>,
{
    fn compare(
        this: &<SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr,
        other: &<SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr,
    ) -> Option<Ordering> {
        match this.len().cmp(&other.len()) {
            Ordering::Greater => {
                if this.keys().all(|key| other.get(key).is_some()) {
                    Some(Ordering::Greater)
                } else {
                    None
                }
            }
            Ordering::Equal => {
                if this.keys().all(|key| other.get(key).is_some()) {
                    Some(Ordering::Equal)
                } else {
                    None
                }
            }
            Ordering::Less => {
                if other.keys().all(|key| this.get(key).is_some()) {
                    Some(Ordering::Less)
                } else {
                    None
                }
            }
        }
    }
}

// impl<Tag: SetTag<T>, T> Debottom for SetUnionRepr<Tag, T>
// where
//     Tag::Bind: Clone,
//     Self::Repr: Collection<T, ()>,
// {
//     fn is_bottom(this: &Self::Repr) -> bool {
//         this.is_empty()
//     }
// }
impl<T: Clone> Debottom for SetUnionRepr<tag::OPTION, T> {
    fn is_bottom(this: &Self::Repr) -> bool {
        this.is_none()
    }

    type DebottomLr = SetUnionRepr<tag::SINGLE, T>;
    fn debottom(this: Self::Repr) -> Option<<Self::DebottomLr as LatticeRepr>::Repr> {
        this.map(crate::lang::collections::Single)
    }
}

fn __assert_merges() {
    use static_assertions::{assert_impl_all, assert_not_impl_any};

    assert_impl_all!(SetUnionRepr<tag::HASH_SET, u32>:
        Merge<SetUnionRepr<tag::HASH_SET, u32>>,
        Merge<SetUnionRepr<tag::BTREE_SET, u32>>,
        Merge<SetUnionRepr<tag::VEC, u32>>,
        Merge<SetUnionRepr<tag::SINGLE, u32>>,
        Merge<SetUnionRepr<tag::OPTION, u32>>,
        Merge<SetUnionRepr<tag::ARRAY<8>, u32>>,
        Merge<SetUnionRepr<tag::MASKED_ARRAY<8>, u32>>,
    );

    assert_impl_all!(SetUnionRepr<tag::BTREE_SET, u32>:
        Merge<SetUnionRepr<tag::HASH_SET, u32>>,
        Merge<SetUnionRepr<tag::BTREE_SET, u32>>,
        Merge<SetUnionRepr<tag::VEC, u32>>,
        Merge<SetUnionRepr<tag::SINGLE, u32>>,
        Merge<SetUnionRepr<tag::OPTION, u32>>,
        Merge<SetUnionRepr<tag::ARRAY<8>, u32>>,
        Merge<SetUnionRepr<tag::MASKED_ARRAY<8>, u32>>,
    );

    assert_impl_all!(SetUnionRepr<tag::VEC, u32>:
        Merge<SetUnionRepr<tag::HASH_SET, u32>>,
        Merge<SetUnionRepr<tag::BTREE_SET, u32>>,
        Merge<SetUnionRepr<tag::VEC, u32>>,
        Merge<SetUnionRepr<tag::SINGLE, u32>>,
        Merge<SetUnionRepr<tag::OPTION, u32>>,
        Merge<SetUnionRepr<tag::ARRAY<8>, u32>>,
        Merge<SetUnionRepr<tag::MASKED_ARRAY<8>, u32>>,
    );

    assert_not_impl_any!(SetUnionRepr<tag::MASKED_ARRAY<8>, u32>:
        Merge<SetUnionRepr<tag::HASH_SET, u32>>,
        Merge<SetUnionRepr<tag::BTREE_SET, u32>>,
        Merge<SetUnionRepr<tag::VEC, u32>>,
        Merge<SetUnionRepr<tag::SINGLE, u32>>,
        Merge<SetUnionRepr<tag::OPTION, u32>>,
        Merge<SetUnionRepr<tag::ARRAY<8>, u32>>,
        Merge<SetUnionRepr<tag::MASKED_ARRAY<8>, u32>>,
    );
}
