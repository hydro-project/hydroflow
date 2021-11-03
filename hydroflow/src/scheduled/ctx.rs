//! Organizational module for Hydroflow Send/RecvCtx structs and Input/OutputPort structs.
use std::cell::RefCell;
use std::rc::Rc;

use crate::scheduled::handoff::{CanReceive, Handoff, TeeingHandoff, TryCanReceive};
use crate::scheduled::OpId;

/**
 * Context provided to a compiled component for writing to an [OutputPort].
 */
pub struct SendCtx<H: Handoff> {
    pub(crate) handoff: Rc<RefCell<H>>,
}
impl<H: Handoff> SendCtx<H> {
    // // TODO: represent backpressure in this return value.
    // #[allow(clippy::result_unit_err)]
    // pub fn give(self, item: H::Item) -> Result<(), ()> {
    //     (*self.once.get()).borrow_mut().try_give(item)
    // }
    pub fn give<T>(&mut self, item: T) -> T
    where
        H: CanReceive<T>,
    {
        let mut borrow = (*self.handoff).borrow_mut();
        <H as CanReceive<T>>::give(&mut *borrow, item)
    }

    pub fn try_give<T>(&mut self, item: T) -> Result<T, T>
    where
        H: TryCanReceive<T>,
    {
        let mut borrow = (*self.handoff).borrow_mut();
        <H as TryCanReceive<T>>::try_give(&mut *borrow, item)
    }
}

/**
 * Handle corresponding to a [SendCtx]. Consumed by [Hydroflow::add_edge] to construct the Hydroflow graph.
 */
#[must_use]
pub struct OutputPort<H: Handoff> {
    pub(crate) op_id: OpId,
    pub(crate) handoff: Rc<RefCell<H>>,
}
impl<H: Handoff> OutputPort<H> {
    pub fn op_id(&self) -> OpId {
        self.op_id
    }
}
impl<T: Clone> Clone for OutputPort<TeeingHandoff<T>> {
    fn clone(&self) -> Self {
        Self {
            op_id: self.op_id,
            handoff: Rc::new(RefCell::new(self.handoff.borrow().tee())),
        }
    }
}

/**
 * Context provided to a compiled component for reading from an [InputPort].
 */
pub struct RecvCtx<H: Handoff> {
    pub(crate) once: Rc<RefCell<Option<Rc<RefCell<H>>>>>,
}
impl<H: Handoff> RecvCtx<H> {
    pub fn take_inner(&mut self) -> H::Inner {
        (*self.once.borrow_mut().as_ref().unwrap().borrow_mut()).take_inner()
    }
}

/**
 * Handle corresponding to a [RecvCtx]. Consumed by [Hydroflow::add_edge] to construct the Hydroflow graph.
 */
// TODO: figure out how to explain succinctly why this and output port both use Writable
#[must_use]
pub struct InputPort<H: Handoff> {
    pub(crate) op_id: OpId,
    pub(crate) once: Rc<RefCell<Option<Rc<RefCell<H>>>>>,
}
impl<H: Handoff> InputPort<H> {
    pub fn op_id(&self) -> OpId {
        self.op_id
    }
}
