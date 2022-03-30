use super::{AssembleFlowGraph, BaseSurface, PullSurface, PushSurface, PushSurfaceReversed};

use crate::builder::build::pull_filter::FilterPullBuild;
use crate::builder::build::push_filter::FilterPushBuild;
use crate::scheduled::context::Context;
use crate::scheduled::flow_graph::NodeId;

pub struct FilterSurface<Prev, Func>
where
    Prev: BaseSurface,
{
    prev: Prev,
    func: Func,
}
impl<Prev, Func> FilterSurface<Prev, Func>
where
    Prev: BaseSurface,
    Func: FnMut(&Context, &Prev::ItemOut) -> bool,
{
    pub fn new(prev: Prev, func: Func) -> Self {
        Self { prev, func }
    }
}

impl<Prev, Func> BaseSurface for FilterSurface<Prev, Func>
where
    Prev: BaseSurface,
    Func: FnMut(&Context, &Prev::ItemOut) -> bool,
{
    type ItemOut = Prev::ItemOut;
}

impl<Prev, Func> PullSurface for FilterSurface<Prev, Func>
where
    Prev: PullSurface,
    Func: FnMut(&Context, &Prev::ItemOut) -> bool,
{
    type InputHandoffs = Prev::InputHandoffs;
    type Build = FilterPullBuild<Prev::Build, Func>;

    fn into_parts(self) -> (Self::InputHandoffs, Self::Build) {
        let (connect, build) = self.prev.into_parts();
        let build = FilterPullBuild::new(build, self.func);
        (connect, build)
    }
}
impl<Prev, Func> AssembleFlowGraph for FilterSurface<Prev, Func>
where
    Prev: PullSurface + AssembleFlowGraph,
    Func: FnMut(&Context, &Prev::ItemOut) -> bool,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> NodeId {
        let my_id = e.add_node("Filter");
        let prev_id = self.prev.insert_dep(e);
        e.add_edge((prev_id, my_id));
        my_id
    }
}

impl<Prev, Func> PushSurface for FilterSurface<Prev, Func>
where
    Prev: PushSurface,
    Func: FnMut(&Context, &Prev::ItemOut) -> bool,
{
    type Output<Next> = Prev::Output<FilterPushSurfaceReversed<Next, Func>>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>;

    fn push_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    {
        self.prev
            .push_to(FilterPushSurfaceReversed::new(next, self.func))
    }
}

pub struct FilterPushSurfaceReversed<Next, Func>
where
    Next: PushSurfaceReversed,
{
    next: Next,
    func: Func,
}
impl<Next, Func> FilterPushSurfaceReversed<Next, Func>
where
    Next: PushSurfaceReversed,
    Func: FnMut(&Context, &Next::ItemIn) -> bool,
{
    pub fn new(next: Next, func: Func) -> Self {
        Self { next, func }
    }
}
impl<Next, Func> AssembleFlowGraph for FilterPushSurfaceReversed<Next, Func>
where
    Next: PushSurfaceReversed + AssembleFlowGraph,
    Func: FnMut(&Context, &Next::ItemIn) -> bool,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> NodeId {
        let my_id = e.add_node("Filter");
        let next_id = self.next.insert_dep(e);
        e.add_edge((my_id, next_id));
        my_id
    }
}

impl<Next, Func> PushSurfaceReversed for FilterPushSurfaceReversed<Next, Func>
where
    Next: PushSurfaceReversed,
    Func: FnMut(&Context, &Next::ItemIn) -> bool,
{
    type ItemIn = Next::ItemIn;

    type OutputHandoffs = Next::OutputHandoffs;
    type Build = FilterPushBuild<Next::Build, Func>;

    fn into_parts(self) -> (Self::OutputHandoffs, Self::Build) {
        let (connect, build) = self.next.into_parts();
        let build = FilterPushBuild::new(build, self.func);
        (connect, build)
    }
}
