use std::task::{Context, Poll};
use std::cell::Cell;

use crate::hide::{Hide, Delta};
use crate::lattice::LatticeRepr;
use crate::metadata::Order;

use super::*;

pub struct OnceOp<Lr: LatticeRepr> {
    value: Cell<Option<Hide<Delta, Lr>>>,
}

impl<Lr: LatticeRepr> PartialEq for OnceOp<Lr> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self as *const Self, other as *const Self)
    }
}
impl<Lr: LatticeRepr> Eq for OnceOp<Lr> {}



impl<Lr: LatticeRepr> OnceOp<Lr> {
    pub fn new(value: Lr::Repr) -> Self {
        Self {
            value: Cell::new(Some(Hide::new(value)))
        }
    }
}

impl<Lr: LatticeRepr> Op for OnceOp<Lr> {
    type LatRepr = Lr;

    fn propegate_saturation(&self) {
    }
}

impl<Lr: LatticeRepr> OpDelta for OnceOp<Lr> {
    type Ord = OnceOrder;

    fn poll_delta(&self, _ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        Poll::Ready(self.value.take())
    }
}

pub struct OnceOrder;
impl Order for OnceOrder {}
