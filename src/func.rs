use std::borrow::Cow;
use crate::lattice::LatticeRepr;
use crate::lattice::pair::PairRepr;
use crate::hide::{Hide, Qualifier, Cumul};
use crate::eight_traits::OpProps;

pub trait Monotone {
    type InLatRepr:  LatticeRepr;
    type OutLatRepr: LatticeRepr;
    fn call<'h, const COMPLETE: bool, const TIME_ORDERED: bool>
        (item: Cow<'h, Hide<Self::InLatRepr, {OpProps { complete: COMPLETE, time_ordered: TIME_ORDERED, lattice_ordered: true}}>>)
        -> Cow<'h, Hide<Self::OutLatRepr, {OpProps { complete: COMPLETE, time_ordered: TIME_ORDERED, lattice_ordered: true}}>>;
}

pub trait Morphism {
    type InLatRepr:  LatticeRepr;
    type OutLatRepr: LatticeRepr;
    fn call<'h, const PROPS: OpProps>
        (item: Cow<'h, Hide<Self::InLatRepr, PROPS>>)
        -> Cow<'h, Hide<Self::OutLatRepr, PROPS>>;
}
pub struct MorphismAsMonotone<F: Morphism + ?Sized> {
    _phantom: std::marker::PhantomData<F>,
}
impl<F: Morphism + ?Sized> Monotone for MorphismAsMonotone<F> {
    type InLatRepr  = F::InLatRepr;
    type OutLatRepr = F::OutLatRepr;
    fn call<'h, const COMPLETE: bool, const TIME_ORDERED: bool>
        (item: Cow<'h, Hide<Self::InLatRepr, {OpProps { complete: COMPLETE, time_ordered: TIME_ORDERED, lattice_ordered: true}}>>)
        -> Cow<'h, Hide<Self::OutLatRepr, {OpProps { complete: COMPLETE, time_ordered: TIME_ORDERED, lattice_ordered: true}}>>
    {
        F::call(item)
    }
}

// pub trait SplitBinaryMorphism {
//     type InLatReprA: LatticeRepr;
//     type InLatReprB: LatticeRepr;
//     type OutLatRepr: LatticeRepr;

//     fn call<'h, Y: Qualifier>(
//         item_a: Cow<'h, Hide<Y, Self::InLatReprA>>,
//         item_b: Cow<'h, Hide<Y, Self::InLatReprB>>,
//     )
//         -> Cow<'h, Hide<Y, Self::OutLatRepr>>;
// }
// pub struct SplitBinaryMorphismAsMonotone<F: SplitBinaryMorphism + ?Sized> {
//     _phantom: std::marker::PhantomData<F>,
// }
// impl<F: SplitBinaryMorphism + ?Sized> Monotone for SplitBinaryMorphismAsMonotone<F> {
//     type InLatRepr  = PairRepr<F::InLatReprA, F::InLatReprB>;
//     type OutLatRepr = F::OutLatRepr;
//     fn call<'h>(item: Cow<'h, Hide<Cumul, Self::InLatRepr>>) -> Cow<'h, Hide<Cumul, Self::OutLatRepr>> {
//         let (a, b) = Hide::split_cow(item);
//         F::call(a, b)
//     }
// }
