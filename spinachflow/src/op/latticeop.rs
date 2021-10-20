use std::cell::RefCell;
use std::task::{Context, Poll};

use crate::hide::{Delta, Hide, Value};
use crate::lattice::{Convert, LatticeRepr, Merge};

use super::*;

/// A state-accumulating lattice op.
pub struct LatticeOp<O: Op, Lr: LatticeRepr + Merge<O::LatRepr>>
where
    O::LatRepr: Convert<Lr>,
{
    op: O,
    state: RefCell<Hide<Value, Lr>>,
}

impl<O: Op, Lr: LatticeRepr + Merge<O::LatRepr>> LatticeOp<O, Lr>
where
    O::LatRepr: Convert<Lr>,
{
    pub fn new(op: O, bottom: Lr::Repr) -> Self {
        Self {
            op,
            state: RefCell::new(Hide::new(bottom)),
        }
    }
}

impl<O: Op, Lr: LatticeRepr + Merge<O::LatRepr>> LatticeOp<O, Lr>
where
    O::LatRepr: Convert<Lr>,
    Lr::Repr: Default,
{
    pub fn new_default(op: O) -> Self {
        Self {
            op,
            state: RefCell::new(Hide::new(Default::default())),
        }
    }
}

impl<O: Op, Lr: LatticeRepr + Merge<O::LatRepr>> Op for LatticeOp<O, Lr>
where
    O::LatRepr: Convert<Lr>,
{
    type LatRepr = Lr;

    fn propegate_saturation(&self) {
        self.op.propegate_saturation()
    }
}

impl<O: OpDelta, Lr: LatticeRepr + Merge<O::LatRepr>> OpDelta for LatticeOp<O, Lr>
where
    O::LatRepr: Convert<Lr>,
{
    type Ord = O::Ord;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        loop {
            match self.op.poll_delta(ctx) {
                Poll::Ready(Some(delta)) => {
                    let state = &mut self.state.borrow_mut();
                    // F::delta(state, &mut delta); // TODO!! Doesn't minimize deltas.
                    if Lr::merge_hide(state, delta.clone()) {
                        return Poll::Ready(Some(<O::LatRepr as Convert<Lr>>::convert_hide(delta)));
                    }
                    // Else: Delta did not change state, try again.
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl<O: Op, Lr: LatticeRepr + Merge<O::LatRepr>> OpValue for LatticeOp<O, Lr>
where
    O::LatRepr: Convert<Lr>,
{
    fn get_value(&self) -> Hide<Value, Self::LatRepr> {
        self.state.borrow().clone()
    }
}
