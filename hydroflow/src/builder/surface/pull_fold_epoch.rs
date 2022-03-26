use super::{AssembleFlowGraph, BaseSurface, PullSurface};

use crate::builder::build::pull_fold_epoch::FoldEpochPullBuild;

use crate::scheduled::context::Context;
use crate::scheduled::graph::NodeId;

pub struct FoldEpochPullSurface<Prev, Init, Func>
where
    Prev: BaseSurface,
{
    prev: Prev,
    init: Init,
    func: Func,
}
impl<Prev, Init, Func, Out> FoldEpochPullSurface<Prev, Init, Func>
where
    Prev: BaseSurface,
    Init: FnMut(&Context<'_>) -> Out,
    Func: FnMut(&Context<'_>, Out, Prev::ItemOut) -> Out,
{
    pub fn new(prev: Prev, init: Init, func: Func) -> Self {
        Self { prev, init, func }
    }
}
impl<Prev, Init, Func, Out> AssembleFlowGraph for FoldEpochPullSurface<Prev, Init, Func>
where
    Prev: BaseSurface + AssembleFlowGraph,
    Init: FnMut(&Context<'_>) -> Out,
    Func: FnMut(&Context<'_>, Out, Prev::ItemOut) -> Out,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> NodeId {
        let my_id = e.add_node("FoldEpoch");
        let prev_id = self.prev.insert_dep(e);
        e.add_edge((prev_id, my_id));
        my_id
    }
}

impl<Prev, Init, Func, Out> BaseSurface for FoldEpochPullSurface<Prev, Init, Func>
where
    Prev: BaseSurface,
    Init: FnMut(&Context<'_>) -> Out,
    Func: FnMut(&Context<'_>, Out, Prev::ItemOut) -> Out,
{
    type ItemOut = Out;
}

impl<Prev, Init, Func, Out> PullSurface for FoldEpochPullSurface<Prev, Init, Func>
where
    Prev: PullSurface,
    Init: FnMut(&Context<'_>) -> Out,
    Func: FnMut(&Context<'_>, Out, Prev::ItemOut) -> Out,
{
    type InputHandoffs = Prev::InputHandoffs;
    type Build = FoldEpochPullBuild<Prev::Build, Init, Func>;

    fn into_parts(self) -> (Self::InputHandoffs, Self::Build) {
        let (connect, build) = self.prev.into_parts();
        let build = FoldEpochPullBuild::new(build, self.init, self.func);
        (connect, build)
    }
}
