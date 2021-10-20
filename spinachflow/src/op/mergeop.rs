use std::task::{Context, Poll};

use crate::hide::{Hide, Delta, Value};
use crate::lattice::{LatticeRepr, Merge, Convert};
use crate::metadata::Order;

use super::*;

pub struct MergeOp<A: Op, B: Op>
where
    A::LatRepr: LatticeRepr<Lattice = <B::LatRepr as LatticeRepr>::Lattice>,
{
    op_a: A,
    op_b: B,
}

impl<A: Op, B: Op> MergeOp<A, B>
where
    A::LatRepr: LatticeRepr<Lattice = <B::LatRepr as LatticeRepr>::Lattice>,
{
    pub fn new(op_a: A, op_b: B) -> Self {
        Self { op_a, op_b }
    }
}

impl<A: Op, B: Op> Op for MergeOp<A, B>
where
    A::LatRepr: LatticeRepr<Lattice = <B::LatRepr as LatticeRepr>::Lattice>,
{
    type LatRepr = A::LatRepr;

    fn propegate_saturation(&self) {
        self.op_a.propegate_saturation();
        self.op_b.propegate_saturation()
    }
}

pub struct MergeOrder<A: Order, B: Order>(std::marker::PhantomData<(A, B)>);
impl<A: Order, B: Order> Order for MergeOrder<A, B> {}

impl<A: OpDelta, B: OpDelta> OpDelta for MergeOp<A, B>
where
    A::LatRepr: LatticeRepr<Lattice = <B::LatRepr as LatticeRepr>::Lattice>,
    B::LatRepr: Convert<A::LatRepr>,
{
    type Ord = MergeOrder<A::Ord, B::Ord>;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        let not_ready = match self.op_a.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => return Poll::Ready(Some(delta)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        };
        match self.op_b.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => Poll::Ready(Some(<B::LatRepr as Convert<A::LatRepr>>::convert_hide(delta))),
            Poll::Ready(None) => not_ready,
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<A: OpValue, B: OpValue> OpValue for MergeOp<A, B>
where
    A::LatRepr: LatticeRepr<Lattice = <B::LatRepr as LatticeRepr>::Lattice>,
    A::LatRepr: Merge<B::LatRepr>,
{
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        let mut val = self.op_a.get_value();
        <A::LatRepr as Merge<B::LatRepr>>::merge_hide(&mut val, self.op_b.get_value());
        val
    }
}
