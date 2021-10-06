use std::task::{Context, Poll};
use std::cell::RefCell;

use crate::hide::{Hide, Delta};
use crate::lattice::LatticeRepr;
use crate::metadata::Order;

use super::*;

pub struct IterOp<Lr: LatticeRepr, I: IntoIterator<Item = Lr::Repr>> {
    iter: RefCell<I::IntoIter>,
    _phantom: std::marker::PhantomData<Lr>,
}

impl<Lr: LatticeRepr, I: IntoIterator<Item = Lr::Repr>> IterOp<Lr, I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter: RefCell::new(iter.into_iter()),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Lr: LatticeRepr, I: IntoIterator<Item = Lr::Repr>> Op for IterOp<Lr, I> {
    type LatRepr = Lr;

    fn propegate_saturation(&self) {
    }
}

impl<Lr: LatticeRepr, I: IntoIterator<Item = Lr::Repr>> OpDelta for IterOp<Lr, I> {
    type Ord = IterOrder;

    fn poll_delta(&self, _ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        Poll::Ready(self.iter.borrow_mut().next().map(Hide::new))
    }
}

pub struct IterOrder;
impl Order for IterOrder {}
