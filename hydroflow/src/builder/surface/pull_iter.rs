use super::{AssembleFlowGraph, BaseSurface, PullSurface};

use crate::builder::build::pull_iter::IterPullBuild;

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
    fn insert_dep(&self, e: &mut super::FlowGraph) -> usize {
        e.add_node("Iter".to_string())
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

    fn into_parts(self) -> (Self::InputHandoffs, Self::Build) {
        ((), IterPullBuild::new(self.it))
    }
}
