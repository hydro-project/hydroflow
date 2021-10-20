use std::task::{Context, Poll};

use crate::hide::{Hide, Delta};
use crate::lattice::{LatticeRepr};
use crate::metadata::Order;

use super::{Op, OpDelta};

pub struct DynOpDelta<Lr: LatticeRepr, Ord: Order> {
    op: Box<dyn OpDelta<LatRepr = Lr, Ord = Ord>>,
}

impl<Lr: LatticeRepr, Ord: Order> DynOpDelta<Lr, Ord> {
    pub fn new(op: Box<dyn OpDelta<LatRepr = Lr, Ord = Ord>>) -> Self {
        Self { op }
    }
}

impl<Lr: LatticeRepr, Ord: Order> Op for DynOpDelta<Lr, Ord> {
    type LatRepr = Lr;

    fn propegate_saturation(&self) {
        self.op.propegate_saturation()
    }
}

impl<Lr: LatticeRepr, Ord: Order> OpDelta for DynOpDelta<Lr, Ord> {
    type Ord = ErasedOrd;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        self.op.poll_delta(ctx)
    }
}

pub enum ErasedOrd {}
impl Order for ErasedOrd {}
