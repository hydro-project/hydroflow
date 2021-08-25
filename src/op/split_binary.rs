use std::borrow::Cow;
use crate::func::SplitBinaryMorphism;
use crate::hide::{Hide, Delta, Cumul};
use crate::lattice::LatticeRepr;
use crate::lattice::null::NullRepr;
use crate::lattice::pair::PairRepr;
use crate::lattice::bottom::BottomRepr;
use super::{Op, OpDelta, OpCumul};

pub struct SplitBinary<PrecOpA, PrecOpB, F>
where
    PrecOpA: OpDelta + OpCumul,
    PrecOpB: OpDelta + OpCumul,
    F: SplitBinaryMorphism<
        InLatReprA = PrecOpA::LatReprDeltaOut,
        InLatReprB = PrecOpB::LatReprDeltaOut,
    >,
{
    _phantom: std::marker::PhantomData<(PrecOpA, PrecOpB, F)>,
}

impl<PrecOpA, PrecOpB, F> Op for SplitBinary<PrecOpA, PrecOpB, F>
where
    PrecOpA: OpDelta + OpCumul,
    PrecOpB: OpDelta + OpCumul,
    F: SplitBinaryMorphism<
        InLatReprA = PrecOpA::LatReprDeltaOut,
        InLatReprB = PrecOpB::LatReprDeltaOut,
    >,
{
    type Lat = <F::OutLatRepr as LatticeRepr>::Lattice;

    type State = PairRepr<PrecOpA::State, PrecOpB::State>;
}

impl<PrecOpA, PrecOpB, F> OpDelta for SplitBinary<PrecOpA, PrecOpB, F>
where
    PrecOpA: OpDelta + OpCumul,
    PrecOpB: OpDelta + OpCumul,
    F: SplitBinaryMorphism<
        InLatReprA = PrecOpA::LatReprDeltaOut,
        InLatReprB = PrecOpB::LatReprDeltaOut,
    >,
{
    type LatReprDeltaIn  = PairRepr<
        BottomRepr<PrecOpA::LatReprDeltaOut>,
        BottomRepr<PrecOpB::LatReprDeltaOut>,
    >;
    type LatReprDeltaOut = F::OutLatRepr;

    fn get_delta<'h>(state: &'h mut Hide<Cumul, Self::State>, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>)
        -> Cow<'h, Hide<Delta, Self::LatReprDeltaOut>>
    {
        let (state_a, state_b) = state.reveal_mut();
        let (el_a, el_b) = Hide::split_cow(element);

        let out_1 = F::call(el_a, PrecOpB::get_cumul(state_b));

        unimplemented!();
    }
}
