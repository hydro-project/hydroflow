use std::fmt::Debug;
use std::task::{Context, Poll};

use crate::hide::{Delta, Hide, Value};
use crate::lattice::LatticeRepr;

use super::*;

pub struct DebugOp<O: Op>
where
    <O::LatRepr as LatticeRepr>::Repr: Debug,
{
    op: O,
    tag: &'static str,
}

impl<O: Op> DebugOp<O>
where
    <O::LatRepr as LatticeRepr>::Repr: Debug,
{
    pub fn new(op: O, tag: &'static str) -> Self {
        Self { op, tag }
    }
}

impl<O: Op> Op for DebugOp<O>
where
    <O::LatRepr as LatticeRepr>::Repr: Debug,
{
    type LatRepr = O::LatRepr;

    fn propegate_saturation(&self) {
        self.op.propegate_saturation()
    }
}

impl<O: OpDelta> OpDelta for DebugOp<O>
where
    <O::LatRepr as LatticeRepr>::Repr: Debug,
{
    type Ord = O::Ord;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        match self.op.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => {
                println!("{} delta: {:?}", self.tag, delta.reveal_ref());
                Poll::Ready(Some(delta))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<O: OpValue> OpValue for DebugOp<O>
where
    <O::LatRepr as LatticeRepr>::Repr: Debug,
{
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        let val = self.op.get_value();
        println!("{} value: {:?}", self.tag, val.reveal_ref());
        val
    }
}
