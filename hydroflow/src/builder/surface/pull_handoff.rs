use super::{BaseSurface, PullSurface};

use std::cell::Cell;
use std::rc::Rc;

use crate::builder::build::pull_handoff::HandoffPullBuild;
use crate::builder::connect::HandoffPullConnect;
use crate::scheduled::ctx::InputPort;
use crate::scheduled::handoff::Handoff;
use crate::tt;

pub struct HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    port: Rc<Cell<Option<InputPort<Hof>>>>,
}

impl<Hof> HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    pub fn new(port: Rc<Cell<Option<InputPort<Hof>>>>) -> Self {
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
    type InputHandoffs = tt!(Hof);

    type Connect = HandoffPullConnect<Hof>;
    type Build = HandoffPullBuild<Hof>;

    fn into_parts(self) -> (Self::Connect, Self::Build) {
        let connect = HandoffPullConnect::new(self.port);
        let build = HandoffPullBuild::new();
        (connect, build)
    }
}
