use super::{AssembleFlowGraph, BaseSurface, PullSurface};

use crate::builder::build::pull_iter::IterPullBuild;
use crate::scheduled::context::Context;
use crate::scheduled::flow_graph::NodeId;

pub struct IterPullSurface<I>
where
    I: Iterator,
{
    it: I,
}

impl<I> IterPullSurface<I>
where
    I: Iterator,
{
    pub fn new(it: I) -> Self {
        Self { it }
    }
}
impl<I> AssembleFlowGraph for IterPullSurface<I>
where
    I: Iterator,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> NodeId {
        e.add_node("Iter")
    }
}

impl<I> BaseSurface for IterPullSurface<I>
where
    I: 'static + Iterator,
{
    type ItemOut = I::Item;
}

impl<I> PullSurface for IterPullSurface<I>
where
    I: 'static + Iterator,
{
    type InputHandoffs = ();
    type Build = IterPullBuild<I>;

    fn into_parts(self, _ctx: &mut Context) -> (Self::InputHandoffs, Self::Build) {
        ((), IterPullBuild::new(self.it))
    }
}
