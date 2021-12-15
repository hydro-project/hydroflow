use super::PullConnect;

use std::cell::Cell;
use std::rc::Rc;

use crate::scheduled::ctx::InputPort;
use crate::scheduled::handoff::{Handoff, HandoffList};
use crate::{tl, tt};

pub struct HandoffPullConnect<Hof>
where
    Hof: Handoff,
{
    port: Rc<Cell<Option<InputPort<Hof>>>>,
}

impl<Hof> HandoffPullConnect<Hof>
where
    Hof: Handoff,
{
    pub fn new(port: Rc<Cell<Option<InputPort<Hof>>>>) -> Self {
        Self { port }
    }
}

impl<Hof> PullConnect for HandoffPullConnect<Hof>
where
    Hof: Handoff,
{
    type InputHandoffs = tt!(Hof);
    fn connect(self, ports: <Self::InputHandoffs as HandoffList>::InputPort) {
        let tl!(port) = ports;
        let old_port = self.port.replace(Some(port));
        assert!(old_port.is_none());
    }
}
