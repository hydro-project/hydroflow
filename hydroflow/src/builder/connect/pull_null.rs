use super::PullConnect;

use std::cell::Cell;
use std::rc::Rc;

use crate::scheduled::ctx::InputPort;
use crate::scheduled::handoff::{Handoff, HandoffList};
use crate::{tl, tt};

pub struct NullPullConnect;

impl PullConnect for NullPullConnect {
    type InputHandoffs = ();
    fn connect(self, ports: <Self::InputHandoffs as HandoffList>::InputPort) {}
}
