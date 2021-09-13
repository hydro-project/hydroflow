use std::iter::FromIterator;
use std::cmp::Ordering;

use super::{Lattice, LatticeRepr, Merge, Convert, Compare, Debottom};

use crate::tag;
use crate::collections::Collection;


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

pub struct SetUnionRepr<Tag: SetTag<T>, T: 'static> {
    _phantom: std::marker::PhantomData<(Tag, T)>,
}

impl<Tag: SetTag<T>, T> LatticeRepr for SetUnionRepr<Tag, T>
where
    Tag::Bind: Clone,
{
    type Lattice = SetUnion<T>;
    type Repr = Tag::Bind;
}

impl<T, SelfTag: SetTag<T>, DeltaTag: SetTag<T>> Merge<SetUnionRepr<DeltaTag, T>> for SetUnionRepr<SelfTag, T>
where
    SetUnionRepr<SelfTag,  T>: LatticeRepr<Lattice = SetUnion<T>>,
    SetUnionRepr<DeltaTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    <SetUnionRepr<SelfTag,  T> as LatticeRepr>::Repr: Collection<T, ()> + Extend<T>,
    <SetUnionRepr<DeltaTag, T> as LatticeRepr>::Repr: IntoIterator<Item = T>,
{
    fn merge(this: &mut <SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr, delta: <SetUnionRepr<DeltaTag, T> as LatticeRepr>::Repr) -> bool {
        let old_len = this.len();
        this.extend(delta);
        this.len() > old_len
    }
}

impl<T, SelfTag: SetTag<T>, TargetTag: SetTag<T>> Convert<SetUnionRepr<TargetTag, T>> for SetUnionRepr<SelfTag, T>
where
    SetUnionRepr<SelfTag,   T>: LatticeRepr<Lattice = SetUnion<T>>,
    SetUnionRepr<TargetTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    <SetUnionRepr<SelfTag,   T> as LatticeRepr>::Repr: IntoIterator<Item = T>,
    <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr: FromIterator<T>,
{
    fn convert(this: <SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr) -> <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr {
        this.into_iter().collect()
    }
}

impl<T: 'static, SelfTag: SetTag<T>, TargetTag: SetTag<T>> Compare<SetUnionRepr<TargetTag, T>> for SetUnionRepr<SelfTag, T>
where
    SetUnionRepr<SelfTag,   T>: LatticeRepr<Lattice = SetUnion<T>>,
    SetUnionRepr<TargetTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
    <SetUnionRepr<SelfTag,   T> as LatticeRepr>::Repr: Collection<T, ()>,
    <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr: Collection<T, ()>,
{
    fn compare(this: &<SetUnionRepr<SelfTag, T> as LatticeRepr>::Repr, other: &<SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr) -> Option<Ordering> {
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
        this.map(crate::collections::Single)
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

mod fns {
    use crate::collections::Single;
    use crate::hide::{Hide};
    use crate::lattice::ord::MaxRepr;
    use crate::props::{OpProps};

    use super::*;

    // // TODO!!!
    // impl<Tag: SetTag<T>, T, const COMPLETE: bool, const TIME_ORDERED: bool>
    //     Hide<SetUnionRepr<Tag, T>, {require_lattice_ordered(COMPLETE, TIME_ORDERED)}>
    // where
    //     Tag::Bind: Clone,
    //     <SetUnionRepr<Tag, T> as LatticeRepr>::Repr: Collection<T, ()>,
    // {
    //     pub fn len(&self) -> Hide<MaxRepr<usize>, {require_lattice_ordered(COMPLETE, TIME_ORDERED)}> {
    //         Hide::new(self.reveal_ref().len())
    //     }
    // }

    impl<Tag: SetTag<T>, T, const META: OpProps> Hide<SetUnionRepr<Tag, T>, META>
    where
        Tag::Bind: Clone,
        <SetUnionRepr<Tag, T> as LatticeRepr>::Repr: Collection<T, ()>,
    {
        pub fn contains(&self, val: &T) -> Hide<MaxRepr<bool>, META> {
            Hide::new(self.reveal_ref().get(val).is_some())
        }
    }

    impl<T, const META: OpProps> Hide<SetUnionRepr<tag::SINGLE, T>, META>
    where
        T: Clone,
    {
        pub fn map_one<U: Clone, F: Fn(T) -> U>(self, f: F) -> Hide<SetUnionRepr<tag::SINGLE, U>, META> {
            Hide::new(crate::collections::Single((f)(self.into_reveal().0)))
        }

        pub fn filter_map_one<U: Clone, F: Fn(T) -> Option<U>>(self, f: F) -> Hide<SetUnionRepr<tag::OPTION, U>, META> {
            Hide::new((f)(self.into_reveal().0))
        }

        pub fn switch_one<F: Fn(&T) -> bool>(self, f: F) -> (Hide<SetUnionRepr<tag::OPTION, T>, META>, Hide<SetUnionRepr<tag::OPTION, T>, META>) {
            let item = self.into_reveal().0;
            if (f)(&item) {
                (Hide::new(Some(item)), Hide::new(None))
            }
            else {
                (Hide::new(None), Hide::new(Some(item)))
            }
        }
    }

    impl<T, const META: OpProps> Hide<SetUnionRepr<tag::OPTION, T>, META>
    where
        T: Clone,
    {
        pub fn map_one<U: Clone, F: Fn(T) -> U>(self, f: F) -> Hide<SetUnionRepr<tag::OPTION, U>, META> {
            Hide::new(self.into_reveal().map(f))
        }

        pub fn filter_map_one<U: Clone, F: Fn(T) -> Option<U>>(self, f: F) -> Hide<SetUnionRepr<tag::OPTION, U>, META> {
            Hide::new(self.into_reveal().and_then(f))
        }

        pub fn switch_one<F: Fn(&T) -> bool>(self, f: F) -> (Hide<SetUnionRepr<tag::OPTION, T>, META>, Hide<SetUnionRepr<tag::OPTION, T>, META>) {
            if let Some(item) = self.into_reveal() {
                if (f)(&item) {
                    (Hide::new(Some(item)), Hide::new(None))
                }
                else {
                    (Hide::new(None), Hide::new(Some(item)))
                }
            }
            else {
                (Hide::new(None), Hide::new(None))
            }
        }
    }

    impl<Tag: SetTag<T>, T, const META: OpProps> Hide<SetUnionRepr<Tag, T>, META>
    where
        SetUnionRepr<Tag, T>: LatticeRepr,
        <SetUnionRepr<Tag, T> as LatticeRepr>::Repr: IntoIterator<Item = T>,
    {
        pub fn filter_map<U, TargetTag: SetTag<U>, F: Fn(T) -> Option<U>>(self, f: F) -> Hide<SetUnionRepr<TargetTag, U>, META>
        where
            SetUnionRepr<TargetTag, U>: LatticeRepr<Lattice = SetUnion<U>>,
            <SetUnionRepr<TargetTag, U> as LatticeRepr>::Repr: FromIterator<U>,
        {
            Hide::new(self.into_reveal().into_iter().filter_map(f).collect())
        }

        pub fn map<U, TargetTag: SetTag<U>, F: Fn(T) -> U>(self, f: F) -> Hide<SetUnionRepr<TargetTag, U>, META>
        where
            SetUnionRepr<TargetTag, U>: LatticeRepr<Lattice = SetUnion<U>>,
            <SetUnionRepr<TargetTag, U> as LatticeRepr>::Repr: FromIterator<U>,
        {
            Hide::new(self.into_reveal().into_iter().map(f).collect())
        }

        pub fn filter<TargetTag: SetTag<T>, F: Fn(&T) -> bool>(self, f: F) -> Hide<SetUnionRepr<TargetTag, T>, META>
        where
            SetUnionRepr<TargetTag, T>: LatticeRepr<Lattice = SetUnion<T>>,
            <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr: FromIterator<T>,
        {
            Hide::new(self.into_reveal().into_iter().filter(f).collect())
        }

        pub fn flatten<TargetTag: SetTag<T::Item>>(self) -> Hide<SetUnionRepr<TargetTag, T::Item>, META>
        where
            T: IntoIterator,
            SetUnionRepr<TargetTag, T::Item>: LatticeRepr<Lattice = SetUnion<T::Item>>,
            <SetUnionRepr<TargetTag, T::Item> as LatticeRepr>::Repr: FromIterator<T::Item>,
        {
            Hide::new(self.into_reveal().into_iter().flatten().collect())
        }

        pub fn switch<TargetTag: SetTag<T>, F: Fn(&T) -> bool>(self, f: F) -> (Hide<SetUnionRepr<TargetTag, T>, META>, Hide<SetUnionRepr<TargetTag, T>, META>)
        where
            T: Clone,
            SetUnionRepr<TargetTag, T>: LatticeRepr<Lattice = SetUnion<T>> + Merge<SetUnionRepr<tag::SINGLE, T>>,
            <SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr: Default,
        {
            let mut out_a = <<SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr as Default>::default();
            let mut out_b = <<SetUnionRepr<TargetTag, T> as LatticeRepr>::Repr as Default>::default();

            for item in self.into_reveal().into_iter() {
                let target = if (f)(&item) { &mut out_a } else { &mut out_b };
                <SetUnionRepr<TargetTag, T> as Merge<SetUnionRepr<tag::SINGLE, T>>>::merge(target, Single(item));
            }

            (Hide::new(out_a), Hide::new(out_b))
        }
    }

    // impl<Tag: SetTag<T>, T, const META: OpProps> Hide<SetUnionRepr<Tag, T>, META>
    // where
    //     SetUnionRepr<Tag, T>: LatticeRepr,
    //     <SetUnionRepr<Tag, T> as LatticeRepr>::Repr: IntoIterator<Item = T>,
    // {
    //     //// CAUSES ICE FOR SOME REASON https://github.com/rust-lang/rust/issues/71113
    //     // pub fn fold<TargetLr, MergeLr>(self) -> Hide<Y, TargetLr>
    //     // where
    //     //     MergeLr: LatticeRepr<Repr = T>,
    //     //     TargetLr: LatticeRepr + Merge<MergeLr>,
    //     //     <TargetLr as LatticeRepr>::Repr: Default,
    //     // {
    //     //     let mut out = Hide::new(Default::default());
    //     //     for t in self.into_reveal().into_iter() {
    //     //         <TargetLr as Merge<MergeLr>>::merge_hide(&mut out, Hide::<Delta, _>::new(t));
    //     //     }
    //     //     out
    //     // }
    // }

    // fn __test_things() {
    //     let my_lattice: Hide<Cumul, SetUnionRepr<tag::HASH_SET, u32>> =
    //         Hide::new(vec![ 0, 1, 2, 3, 5, 8, 13 ].into_iter().collect());

    //     let _: Hide<Cumul, MaxRepr<usize>> = my_lattice.len();
    //     let _: Hide<Cumul, MaxRepr<bool>>  = my_lattice.contains(&4);

    //     let my_delta: Hide<Delta, SetUnionRepr<tag::HASH_SET, u32>> =
    //         Hide::new(vec![ 0, 1, 2, 3, 5, 8, 13 ].into_iter().collect());

    //     // let _: Hide<Cumul, MaxRepr<usize>> = my_delta.len();
    //     let _: Hide<Cumul, MaxRepr<bool>>  = my_delta.contains(&4);
    // }
}
