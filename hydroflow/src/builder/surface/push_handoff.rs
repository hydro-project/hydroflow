use super::PushSurfaceReversed;

use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::builder::build::push_handoff::HandoffPushBuild;
use crate::builder::connect::HandoffPushConnect;
use crate::scheduled::ctx::OutputPort;
use crate::scheduled::handoff::{CanReceive, Handoff};
use crate::tt;

pub struct HandoffPushSurfaceReversed<Hof, In>
where
    Hof: Handoff + CanReceive<In>,
{
    port: Rc<Cell<Option<OutputPort<Hof>>>>,
    _phantom: PhantomData<fn(In)>,
}

impl<Hof, In> HandoffPushSurfaceReversed<Hof, In>
where
    Hof: Handoff + CanReceive<In>,
{
    pub fn new(port: Rc<Cell<Option<OutputPort<Hof>>>>) -> Self {
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
    type OutputHandoffs = tt!(Hof);

    type ItemIn = In;

    type Connect = HandoffPushConnect<Hof>;
    type Build = HandoffPushBuild<Hof, In>;

    fn into_parts(self) -> (Self::Connect, Self::Build) {
        let connect = HandoffPushConnect::new(self.port);
        let build = HandoffPushBuild::new();
        (connect, build)
    }
}
