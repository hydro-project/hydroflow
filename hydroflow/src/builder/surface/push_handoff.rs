use super::PushSurfaceReversed;

use std::marker::PhantomData;

use crate::builder::build::push_handoff::HandoffPushBuild;
use crate::scheduled::handoff::{CanReceive, Handoff};
use crate::scheduled::port::OutputPort;
use crate::tt;

pub struct HandoffPushSurfaceReversed<Hof, In>
where
    Hof: Handoff + CanReceive<In>,
{
    port: Option<OutputPort<Hof>>,
    _phantom: PhantomData<fn(In)>,
}

impl<Hof, In> HandoffPushSurfaceReversed<Hof, In>
where
    Hof: Handoff + CanReceive<In>,
{
    pub fn new(port: OutputPort<Hof>) -> Self {
        Self {
            port: Some(port),
            _phantom: PhantomData,
        }
    }
}

impl<Hof, In> PushSurfaceReversed for HandoffPushSurfaceReversed<Hof, In>
where
    Hof: Handoff + CanReceive<In>,
{
    type OutputHandoffs = tt!(Hof);

    type ItemIn = In;

    type Build = HandoffPushBuild<Hof, In>;

    fn into_build(self) -> Self::Build {
        HandoffPushBuild::new()
    }
}
