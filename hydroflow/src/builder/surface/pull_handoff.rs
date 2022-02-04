use super::{BaseSurface, PullSurface};

use crate::builder::build::pull_handoff::HandoffPullBuild;
use crate::scheduled::handoff::Handoff;
use crate::scheduled::port::RecvPort;
use crate::{tl, tt};

pub struct HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    port: RecvPort<Hof>,
}

impl<Hof> HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    pub fn new(port: RecvPort<Hof>) -> Self {
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
    type InputHandoffs = tt!(RecvPort<Hof>);
    type Build = HandoffPullBuild<Hof>;

    fn into_parts(self) -> (Self::InputHandoffs, Self::Build) {
        (tl!(self.port), HandoffPullBuild::new())
    }
}
