use super::PushSurfaceReversed;

use std::marker::PhantomData;

use crate::builder::build::push_handoff::HandoffPushBuild;
use crate::scheduled::handoff::{CanReceive, Handoff};
use crate::scheduled::port::InputPort;
use crate::{tl, tt};

pub struct HandoffPushSurfaceReversed<Hof, In>
where
    Hof: Handoff + CanReceive<In>,
{
    port: InputPort<Hof>,
    _phantom: PhantomData<fn(In)>,
}

impl<Hof, In> HandoffPushSurfaceReversed<Hof, In>
where
    Hof: Handoff + CanReceive<In>,
{
    pub fn new(port: InputPort<Hof>) -> Self {
        Self {
            port,
            _phantom: PhantomData,
        }
    }
}

impl<Hof, In> PushSurfaceReversed for HandoffPushSurfaceReversed<Hof, In>
where
    Hof: Handoff + CanReceive<In>,
{
    type ItemIn = In;

    type OutputHandoffs = tt!(InputPort<Hof>);
    type Build = HandoffPushBuild<Hof, In>;

    fn into_parts(self) -> (Self::OutputHandoffs, Self::Build) {
        (tl!(self.port), HandoffPushBuild::new())
    }
}
