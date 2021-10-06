use std::iter::FromIterator;
use std::cmp::Ordering;

use super::{Lattice, LatticeRepr, Merge, Convert, Compare, Debottom};

use crate::tag;
use crate::collections::Collection;


pub trait Order {}
pub enum IdOrder<const ID: u64> {}
impl<const ID: u64> Order for IdOrder<ID> {}

pub struct Sequence<T, O: Order> {
    _phantom: std::marker::PhantomData<(T, O)>,
}
impl<T, O: Order> Lattice for Sequence<T, O> {}

pub trait SequenceTag<T>: tag::Tag1<T> {}
impl<T> SequenceTag<T> for tag::VEC {}
impl<T> SequenceTag<T> for tag::SINGLE {}
impl<T> SequenceTag<T> for tag::OPTION {}
impl<T, const N: usize> SequenceTag<T> for tag::ARRAY<N> {}
impl<T, const N: usize> SequenceTag<T> for tag::MASKED_ARRAY<N> {}

pub struct SequenceRepr<Tag: SequenceTag<T>, T: 'static, O: Order> {
    _phantom: std::marker::PhantomData<(Tag, T, O)>,
}

impl<Tag: SequenceTag<T>, T, O: Order + 'static> LatticeRepr for SequenceRepr<Tag, T, O>
where
    Tag::Bind: Clone,
{
    type Lattice = Sequence<T, O>;
    type Repr = Tag::Bind;
}

impl<T, O: Order, SelfTag: SequenceTag<T>, DeltaTag: SequenceTag<T>> Merge<SequenceRepr<DeltaTag, T, O>> for SequenceRepr<SelfTag, T, O>
where
    SequenceRepr<SelfTag,  T, O>: LatticeRepr<Lattice = Sequence<T, O>>,
    SequenceRepr<DeltaTag, T, O>: LatticeRepr<Lattice = Sequence<T, O>>,
    <SequenceRepr<SelfTag,  T, O> as LatticeRepr>::Repr: Collection<T, ()> + Extend<T>,
    <SequenceRepr<DeltaTag, T, O> as LatticeRepr>::Repr: IntoIterator<Item = T>,
{
    fn merge(this: &mut <SequenceRepr<SelfTag, T, O> as LatticeRepr>::Repr, delta: <SequenceRepr<DeltaTag, T, O> as LatticeRepr>::Repr) -> bool {
        let old_len = this.len();
        this.extend(delta);
        this.len() > old_len
    }
}

impl<T, O: Order, SelfTag: SequenceTag<T>, TargetTag: SequenceTag<T>> Convert<SequenceRepr<TargetTag, T, O>> for SequenceRepr<SelfTag, T, O>
where
    SequenceRepr<SelfTag,   T, O>: LatticeRepr<Lattice = Sequence<T, O>>,
    SequenceRepr<TargetTag, T, O>: LatticeRepr<Lattice = Sequence<T, O>>,
    <SequenceRepr<SelfTag,   T, O> as LatticeRepr>::Repr: IntoIterator<Item = T>,
    <SequenceRepr<TargetTag, T, O> as LatticeRepr>::Repr: FromIterator<T>,
{
    fn convert(this: <SequenceRepr<SelfTag, T, O> as LatticeRepr>::Repr) -> <SequenceRepr<TargetTag, T, O> as LatticeRepr>::Repr {
        this.into_iter().collect()
    }
}

impl<T: 'static, O: Order, SelfTag: SequenceTag<T>, TargetTag: SequenceTag<T>> Compare<SequenceRepr<TargetTag, T, O>> for SequenceRepr<SelfTag, T, O>
where
    SequenceRepr<SelfTag,   T, O>: LatticeRepr<Lattice = Sequence<T, O>>,
    SequenceRepr<TargetTag, T, O>: LatticeRepr<Lattice = Sequence<T, O>>,
    <SequenceRepr<SelfTag,   T, O> as LatticeRepr>::Repr: Collection<T, ()>,
    <SequenceRepr<TargetTag, T, O> as LatticeRepr>::Repr: Collection<T, ()>,
{
    fn compare(this: &<SequenceRepr<SelfTag, T, O> as LatticeRepr>::Repr, other: &<SequenceRepr<TargetTag, T, O> as LatticeRepr>::Repr) -> Option<Ordering> {
        if this.len() > other.len() {
            if this.keys().all(|key| other.get(key).is_some()) {
                Some(Ordering::Greater)
            }
            else {
                None
            }
        }
        else if this.len() == other.len() {
            if this.keys().all(|key| other.get(key).is_some()) {
                Some(Ordering::Equal)
            }
            else {
                None
            }
        }
        else { // this.len() < other.len()
            if other.keys().all(|key| this.get(key).is_some()) {
                Some(Ordering::Less)
            }
            else {
                None
            }
        }
    }
}

// impl<Tag: SequenceTag<T>, T> Debottom for SequenceRepr<Tag, T>
// where
//     Tag::Bind: Clone,
//     Self::Repr: Collection<T, ()>,
// {
//     fn is_bottom(this: &Self::Repr) -> bool {
//         this.is_empty()
//     }
// }
impl<T: Clone, O: Order + 'static> Debottom for SequenceRepr<tag::OPTION, T, O> {
    fn is_bottom(this: &Self::Repr) -> bool {
        this.is_none()
    }

    type DebottomLr = SequenceRepr<tag::SINGLE, T, O>;
    fn debottom(this: Self::Repr) -> Option<<Self::DebottomLr as LatticeRepr>::Repr> {
        this.map(crate::collections::Single)
    }
}
