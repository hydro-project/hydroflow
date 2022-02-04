//! Organizational module for Hydroflow Send/RecvCtx structs and Input/OutputPort structs.
use std::marker::PhantomData;

use ref_cast::RefCast;
use sealed::sealed;

use crate::scheduled::handoff::{CanReceive, Handoff, TryCanReceive};

use super::HandoffId;

#[sealed]
pub trait Polarity: 'static {}

pub enum SEND {}
pub enum RECV {}
#[sealed]
impl Polarity for SEND {}
#[sealed]
impl Polarity for RECV {}

#[must_use]
pub struct Port<S: Polarity, H>
where
    H: Handoff,
{
    pub(crate) handoff_id: HandoffId,
    #[allow(clippy::type_complexity)]
    pub(crate) _marker: PhantomData<(*const S, fn() -> H)>,
}
pub type InputPort<H> = Port<SEND, H>;
pub type OutputPort<H> = Port<RECV, H>;

#[derive(RefCast)]
#[repr(transparent)]
pub struct PortCtx<S: Polarity, H> {
    pub(crate) handoff: H,
    pub(crate) _marker: PhantomData<*const S>,
}

pub type SendCtx<H> = PortCtx<SEND, H>;
pub type RecvCtx<H> = PortCtx<RECV, H>;

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
