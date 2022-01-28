//! Organizational module for Hydroflow Send/RecvCtx structs and Input/OutputPort structs.
use std::marker::PhantomData;

use ref_cast::RefCast;

use crate::scheduled::handoff::{CanReceive, Handoff, TryCanReceive};

use super::HandoffId;

#[must_use]
pub struct BasePort<H, const S: bool>
where
    H: Handoff,
{
    pub(crate) handoff_id: HandoffId,
    pub(crate) _marker: PhantomData<fn() -> H>,
}

pub type InputPort<H> = BasePort<H, true>; // SendPort
pub type OutputPort<H> = BasePort<H, false>; // RecvPort

#[derive(RefCast)]
#[repr(transparent)]
pub struct BaseCtx<H, const S: bool> {
    pub(crate) handoff: H,
}

pub type SendCtx<H> = BaseCtx<H, true>;
pub type RecvCtx<H> = BaseCtx<H, false>;

/// Context provided to a compiled component for writing to an [OutputPort].
impl<H: Handoff> SendCtx<H> {
    // TODO: represent backpressure in this return value.
    pub fn give<T>(&self, item: T) -> T
    where
        H: CanReceive<T>,
    {
        <H as CanReceive<T>>::give(&self.handoff, item)
    }

    pub fn try_give<T>(&self, item: T) -> Result<T, T>
    where
        H: TryCanReceive<T>,
    {
        <H as TryCanReceive<T>>::try_give(&self.handoff, item)
    }
}

/// Context provided to a compiled component for reading from an [InputPort].
impl<H: Handoff> RecvCtx<H> {
    pub fn take_inner(&self) -> H::Inner {
        self.handoff.take_inner()
    }
}
