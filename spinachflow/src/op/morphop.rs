use std::task::{Context, Poll};

use crate::func::unary::Morphism;
use crate::hide::{Delta, Hide, Value};
use crate::metadata::Order;

use super::*;

pub struct MorphismOp<O: Op, F: Morphism<InLatRepr = O::LatRepr>> {
    op: O,
    func: F,
}

impl<O: Op, F: Morphism<InLatRepr = O::LatRepr>> MorphismOp<O, F> {
    pub fn new(op: O, func: F) -> Self {
        Self { op, func }
    }
}

impl<O: Op, F: Morphism<InLatRepr = O::LatRepr>> Op for MorphismOp<O, F> {
    type LatRepr = F::OutLatRepr;

    fn propegate_saturation(&self) {
        self.op.propegate_saturation()
    }
}

impl<O: OpDelta, F: Morphism<InLatRepr = O::LatRepr>> OpDelta for MorphismOp<O, F> {
    type Ord = MorphismOrder<O::Ord, F>;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        match self.op.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => Poll::Ready(Some(self.func.call(delta))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<O: OpValue, F: Morphism<InLatRepr = O::LatRepr>> OpValue for MorphismOp<O, F> {
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        self.func.call(self.op.get_value())
    }
}

pub struct MorphismOrder<O: Order, F: Morphism>(std::marker::PhantomData<(O, F)>);
impl<O: Order, F: Morphism> Order for MorphismOrder<O, F> {}
