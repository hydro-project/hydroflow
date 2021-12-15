use super::PushConnect;

use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::scheduled::ctx::OutputPort;
use crate::scheduled::handoff::{Handoff, HandoffList};
use crate::{tl, tt};

pub struct HandoffPushConnect<Hof>
where
    Hof: Handoff,
{
    port: Rc<Cell<Option<OutputPort<Hof>>>>,
    _phantom: PhantomData<fn(Hof)>,
}

impl<Hof> HandoffPushConnect<Hof>
where
    Hof: Handoff,
{
    pub fn new(port: Rc<Cell<Option<OutputPort<Hof>>>>) -> Self {
        Self {
            port,
            _phantom: PhantomData,
        }
    }
}

impl<Hof> PushConnect for HandoffPushConnect<Hof>
where
    Hof: Handoff,
{
    type OutputHandoffs = tt!(Hof);
    fn connect(self, ports: <Self::OutputHandoffs as HandoffList>::OutputPort) {
        let tl!(port) = ports;
        let old_port = self.port.replace(Some(port));
        assert!(old_port.is_none());
    }
}
