use super::{BaseSurface, PullSurface};

use std::cell::Cell;
use std::rc::Rc;

use crate::builder::build::pull_handoff::HandoffPullBuild;
use crate::builder::build::pull_iter::IterPullBuild;
use crate::builder::connect::{HandoffPullConnect, NullPullConnect};
use crate::scheduled::ctx::InputPort;
use crate::scheduled::handoff::Handoff;
use crate::tt;

pub struct IterPullSurface<I, T>
where
    I: Iterator<Item = T>,
{
    it: I,
}

impl<I, T> IterPullSurface<I, T>
where
    I: Iterator<Item = T>,
{
    pub fn new(it: I) -> Self {
        Self { it }
    }
}

impl<I, T> BaseSurface for IterPullSurface<I, T>
where
    I: Iterator<Item = T>,
{
    type ItemOut = T;
}

impl<I, T> PullSurface for IterPullSurface<I, T>
where
    I: 'static + Iterator<Item = T>,
{
    type InputHandoffs = ();

    type Connect = NullPullConnect;
    type Build = IterPullBuild<I, T>;

    fn into_parts(self) -> (Self::Connect, Self::Build) {
        let connect = NullPullConnect;
        let build = IterPullBuild::new(self.it);
        (connect, build)
    }
}
