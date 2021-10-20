use std::task::{Context, Poll};

use crate::hide::{Hide, Delta, Value};
use crate::lattice::LatticeRepr;

use super::*;

pub struct ConstOp<Lr: LatticeRepr> {
    value: Lr::Repr,
}

impl<Lr: LatticeRepr> ConstOp<Lr> {
    pub fn new(value: Lr::Repr) -> Self {
        Self { value }
    }
}

impl<Lr: LatticeRepr> Op for ConstOp<Lr> {
    type LatRepr = Lr;

    fn propegate_saturation(&self) {
    }
}

impl<Lr: LatticeRepr> OpDelta for ConstOp<Lr> {
    type Ord = crate::metadata::EmptyOrder;

    fn poll_delta(&self, _ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        Poll::Ready(None)
    }
}

impl<Lr: LatticeRepr> OpValue for ConstOp<Lr> {
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        Hide::new(self.value.clone())
    }
}
