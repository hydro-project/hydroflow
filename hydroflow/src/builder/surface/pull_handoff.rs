use super::{BaseSurface, PullSurface};

use crate::builder::build::pull_handoff::HandoffPullBuild;
use crate::scheduled::handoff::Handoff;
use crate::scheduled::port::InputPort;
use crate::tt;

pub struct HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    port: Option<InputPort<Hof>>,
}

impl<Hof> HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    pub fn new(port: InputPort<Hof>) -> Self {
        Self { port: Some(port) }
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
    type InputHandoffs = tt!(Hof);

    type Build = HandoffPullBuild<Hof>;

    fn into_build(self) -> Self::Build {
        HandoffPullBuild::new()
    }
}
