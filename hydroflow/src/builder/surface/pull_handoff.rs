use super::{BaseSurface, PullSurface};

use crate::builder::build::pull_handoff::HandoffPullBuild;
use crate::scheduled::handoff::Handoff;
use crate::scheduled::port::OutputPort;
use crate::{tl, tt};

pub struct HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    port: OutputPort<Hof>,
}

impl<Hof> HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    pub fn new(port: OutputPort<Hof>) -> Self {
        Self { port }
    }
}

impl<Hof> BaseSurface for HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    type ItemOut = Hof::Inner;
}

impl<Hof> PullSurface for HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    type InputHandoffs = tt!(OutputPort<Hof>);
    type Build = HandoffPullBuild<Hof>;

    fn into_parts(self) -> (Self::InputHandoffs, Self::Build) {
        (tl!(self.port), HandoffPullBuild::new())
    }
}
