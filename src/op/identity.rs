use std::borrow::Cow;
use crate::hide::{Hide, Delta, Cumul};
use crate::lattice::LatticeRepr;
use crate::lattice::null::NullRepr;
use super::{Op, OpDelta};

pub struct Identity<Lr: LatticeRepr> {
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: LatticeRepr> Op for Identity<Lr> {
    type ILatRepr = Lr;
    type OLatRepr = Lr;

    type State = NullRepr;
}

impl<OLr: LatticeRepr> OpDelta for Identity<OLr> {
    fn get_delta<'h>(_state: &'h mut Hide<Cumul, Self::State>, element: Cow<'h, Hide<Delta, Self::ILatRepr>>)
        -> Cow<'h, Hide<Delta, Self::OLatRepr>>
    {
        element
    }
}
