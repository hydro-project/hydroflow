use std::cell::RefCell;
use std::task::{Context, Poll};

use tokio::sync::mpsc;

use crate::lattice::LatticeRepr;
use crate::hide::{Hide, Delta};
use crate::metadata::Order;

use super::optrait::*;

pub struct ChannelOp<Lr: LatticeRepr> {
    receiver: RefCell<mpsc::UnboundedReceiver<Hide<Delta, Lr>>>,
}

impl<Lr: LatticeRepr> ChannelOp<Lr>
{
    pub fn new(receiver: mpsc::UnboundedReceiver<Hide<Delta, Lr>>) -> Self {
        Self {
            receiver: RefCell::new(receiver),
        }
    }
}

impl<Lr: LatticeRepr> Op for ChannelOp<Lr> {
    type LatRepr = Lr;

    fn propegate_saturation(&self) {
        unimplemented!("TODO");
    }
}

pub enum ChannelOrder {}
impl Order for ChannelOrder {}

impl<Lr: LatticeRepr> OpDelta for ChannelOp<Lr> {
    type Ord = ChannelOrder;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        self.receiver.borrow_mut().poll_recv(ctx)
    }
}
