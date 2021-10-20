use std::task::{Context, Poll};

use crate::hide::{Delta, Hide, Value};
use crate::lattice::{Convert, LatticeRepr};

use super::*;

pub struct BatchConvertOp<O: Op, Lr>
where
    Lr: LatticeRepr<Lattice = <O::LatRepr as LatticeRepr>::Lattice>,
    O::LatRepr: Convert<Lr>,
{
    op: O,
    _phantom: std::marker::PhantomData<Lr>,
}

impl<O: Op, Lr> BatchConvertOp<O, Lr>
where
    Lr: LatticeRepr<Lattice = <O::LatRepr as LatticeRepr>::Lattice>,
    O::LatRepr: Convert<Lr>,
{
    pub fn new(op: O) -> Self {
        Self {
            op,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<O: Op, Lr> Op for BatchConvertOp<O, Lr>
where
    Lr: LatticeRepr<Lattice = <O::LatRepr as LatticeRepr>::Lattice>,
    O::LatRepr: Convert<Lr>,
{
    type LatRepr = Lr;

    fn propegate_saturation(&self) {
        self.op.propegate_saturation()
    }
}

impl<O: OpDelta, Lr> OpDelta for BatchConvertOp<O, Lr>
where
    Lr: LatticeRepr<Lattice = <O::LatRepr as LatticeRepr>::Lattice>,
    O::LatRepr: Convert<Lr>,
{
    type Ord = O::Ord;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        match self.op.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => {
                Poll::Ready(Some(<O::LatRepr as Convert<Lr>>::convert_hide(delta)))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<O: OpValue, Lr> OpValue for BatchConvertOp<O, Lr>
where
    Lr: LatticeRepr<Lattice = <O::LatRepr as LatticeRepr>::Lattice>,
    O::LatRepr: Convert<Lr>,
{
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        <O::LatRepr as Convert<Lr>>::convert_hide(self.op.get_value())
    }
}
