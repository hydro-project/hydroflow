use std::cell::Cell;
use std::task::{Context, Poll};

use crate::hide::{Hide, Delta, Value};
use crate::lattice::{Top};

use super::*;

pub struct TopOp<O: Op>
where
    O::LatRepr: Top,
{
    op: O,
    at_top: Cell<bool>,
}

impl<O: Op> TopOp<O>
where
    O::LatRepr: Top,
{
    pub fn new(op: O) -> Self {
        Self {
            op,
            at_top: Cell::default(),
        }
    }
}

impl<O: Op> Op for TopOp<O>
where
    O::LatRepr: Top,
{
    type LatRepr = O::LatRepr;

    fn propegate_saturation(&self) {
        self.at_top.replace(true);
        self.op.propegate_saturation()
    }
}

impl<O: OpDelta> OpDelta for TopOp<O>
where
    O::LatRepr: Top,
{
    type Ord = O::Ord;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        if self.at_top.get() {
            Poll::Ready(None)
            // TODO?
            // Poll::Ready(Some(Hide::new(<Self::LatRepr as Top>::top())))
        }
        else {
            match self.op.poll_delta(ctx) {
                Poll::Ready(Some(delta)) => {
                    self.propegate_saturation();
                    Poll::Ready(Some(delta))
                }
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

impl<O: OpValue> OpValue for TopOp<O>
where
    O::LatRepr: Top,
{
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        if self.at_top.get() {
            Hide::new(<Self::LatRepr as Top>::top())
        }
        else {
            self.op.get_value()
        }
    }
}
