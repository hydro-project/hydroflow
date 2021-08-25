use std::borrow::Cow;
use crate::hide::{Hide, Delta, Cumul};
use crate::lattice::LatticeRepr;
use crate::lattice::null::NullRepr;
use super::{Op, OpDelta};

pub struct Identity<Lr: LatticeRepr> {
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: LatticeRepr> Op for Identity<Lr> {
    type Lat = Lr::Lattice;

    type State = NullRepr;
}

impl<Lr: LatticeRepr> OpDelta for Identity<Lr> {
    type LatReprDeltaIn = Lr;
    type LatReprDeltaOut = Lr;

    fn get_delta<'h>(_state: &'h mut Hide<Cumul, Self::State>, element: Cow<'h, Hide<Delta, Self::LatReprDeltaIn>>)
        -> Cow<'h, Hide<Delta, Self::LatReprDeltaOut>> {
        element
    }
}
