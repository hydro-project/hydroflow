use std::borrow::Cow;
use crate::hide::{Hide, Delta, Cumul};
use crate::lattice::{LatticeRepr};

use super::{Op, OpDelta};

pub struct Debug<PrecOp>
where
    PrecOp: OpDelta,
    <PrecOp::LatReprDeltaOut as LatticeRepr>::Repr: std::fmt::Debug,
{
    _phantom: std::marker::PhantomData<PrecOp>,
}

impl<PrecOp> Op for Debug<PrecOp>
where
    PrecOp: OpDelta,
    <PrecOp::LatReprDeltaOut as LatticeRepr>::Repr: std::fmt::Debug,
{
    type Lat = <PrecOp::LatReprDeltaOut as LatticeRepr>::Lattice;

    type State = PrecOp::State;
}

impl<PrecOp> OpDelta for Debug<PrecOp>
where
    PrecOp: OpDelta,
    <PrecOp::LatReprDeltaOut as LatticeRepr>::Repr: std::fmt::Debug,
{
    type LatReprDeltaIn = PrecOp::LatReprDeltaIn;
    type LatReprDeltaOut = PrecOp::LatReprDeltaOut;

    fn get_delta<'h>(state: &'h mut Hide<Cumul, Self::State>, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>)
        -> Cow<'h, Hide<Delta, Self::LatReprDeltaOut>>
    {
        let element = PrecOp::get_delta(state, element);
        println!("DEBUG OP: {:#?}", element.reveal_ref());
        return element;
    }
}

// impl<PrecOp, Lr> OpCumul for Debug<PrecOp, Lr>
// where
//     PrecOp: OpDelta,
//     Lr: LatticeRepr + Merge<PrecOp::LatReprDeltaOut>,
// {
//     type LatReprCumulOut = Lr;

//     fn get_cumul<'h>(state: &'h mut Hide<Cumul, Self::State>)
//         -> Cow<'h, Hide<Cumul, Self::LatReprCumulOut>>
//     {
//         let (_prec_state, self_state) = state.split_mut();
//         return Cow::Borrowed(&*self_state);
//     }
// }