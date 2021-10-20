use std::task::{Context, Poll};

use crate::hide::{Delta, Hide, Value};
use crate::lattice::LatticeRepr;
use crate::metadata::Order;

pub trait Op {
    /// The output element type of this op.
    type LatRepr: LatticeRepr;

    /// Top saturation.
    fn propegate_saturation(&self);
}

pub trait OpDelta: Op {
    /// Ordering metadata.
    type Ord: Order;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>>;
}

pub trait OpValue: Op {
    fn get_value(&self) -> Hide<Value, Self::LatRepr>;
}
