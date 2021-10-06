use std::task::{Context, Poll};

use crate::hide::{Hide, Delta};
use crate::lattice::{Debottom};

use super::*;

pub struct DebottomOp<O: Op>
where
    O::LatRepr: Debottom,
{
    op: O,
}

impl<O: Op> DebottomOp<O>
where
    O::LatRepr: Debottom,
{
    pub fn new(op: O) -> Self {
        Self { op }
    }
}

impl<O: Op> Op for DebottomOp<O>
where
    O::LatRepr: Debottom,
{
    type LatRepr = <O::LatRepr as Debottom>::DebottomLr;

    fn propegate_saturation(&self) {
        self.op.propegate_saturation()
    }
}

impl<O: OpDelta> OpDelta for DebottomOp<O>
where
    O::LatRepr: Debottom,
{
    type Ord = O::Ord;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        loop {
            return match self.op.poll_delta(ctx) {
                Poll::Ready(Some(delta)) => {
                    match O::LatRepr::debottom(delta.into_reveal()) {
                        Some(non_bottom_repr) => Poll::Ready(Some(Hide::new(non_bottom_repr))),
                        None => continue,
                    }
                }
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}
