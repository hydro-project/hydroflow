use std::task::{Context, Poll};

use crate::func::binary::BinaryMorphism;
use crate::hide::{Hide, Delta, Value};
use crate::metadata::Order;

use super::*;

pub struct BinaryOp<A: OpValue, B: OpValue, F>
where
    F: BinaryMorphism<InLatReprA = A::LatRepr, InLatReprB = B::LatRepr>,
{
    op_a: A,
    op_b: B,
    func: F,
}

impl<A: OpValue, B: OpValue, F> BinaryOp<A, B, F>
where
    F: BinaryMorphism<InLatReprA = A::LatRepr, InLatReprB = B::LatRepr>,
{
    pub fn new(op_a: A, op_b: B, func: F) -> Self {
        Self { op_a, op_b, func }
    }
}

impl<A: OpValue, B: OpValue, F> Op for BinaryOp<A, B, F>
where
F: BinaryMorphism<InLatReprA = A::LatRepr, InLatReprB = B::LatRepr>,
{
    type LatRepr = F::OutLatRepr;

    fn propegate_saturation(&self) {
        self.op_a.propegate_saturation();
        self.op_b.propegate_saturation()
    }
}

pub struct BinaryOpOrder<A: Order, B: Order, F>(std::marker::PhantomData<(A, B, F)>);
impl<A: Order, B: Order, F> Order for BinaryOpOrder<A, B, F> {}

impl<A: OpValue + OpDelta, B: OpValue + OpDelta, F> OpDelta for BinaryOp<A, B, F>
where
    F: BinaryMorphism<InLatReprA = A::LatRepr, InLatReprB = B::LatRepr>,
{
    type Ord = BinaryOpOrder<A::Ord, B::Ord, F>;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        let not_ready = match self.op_a.poll_delta(ctx) {
            Poll::Ready(Some(delta_a)) => {
                let out = self.func.call(delta_a, self.op_b.get_value().into_delta());
                return Poll::Ready(Some(out));
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        };
        match self.op_b.poll_delta(ctx) {
            Poll::Ready(Some(delta_b)) => {
                let out = self.func.call(self.op_a.get_value().into_delta(), delta_b);
                return Poll::Ready(Some(out));
            }
            Poll::Ready(None) => not_ready,
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<A: OpValue, B: OpValue, F> OpValue for BinaryOp<A, B, F>
where
    F: BinaryMorphism<InLatReprA = A::LatRepr, InLatReprB = B::LatRepr>,
{
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        self.func.call(self.op_a.get_value(), self.op_b.get_value())
    }
}
