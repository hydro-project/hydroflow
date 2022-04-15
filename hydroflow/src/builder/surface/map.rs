use super::{AssembleFlowGraph, BaseSurface, PullSurface, PushSurface, PushSurfaceReversed};

use std::marker::PhantomData;

use crate::builder::build::pull_map::MapPullBuild;
use crate::builder::build::push_map::MapPushBuild;
use crate::scheduled::context::Context;
use crate::scheduled::flow_graph::NodeId;

pub struct MapSurface<Prev, Func>
where
    Prev: BaseSurface,
{
    prev: Prev,
    func: Func,
}
impl<Prev, Func, Out> MapSurface<Prev, Func>
where
    Prev: BaseSurface,
    Func: FnMut(&Context, Prev::ItemOut) -> Out,
{
    pub fn new(prev: Prev, func: Func) -> Self {
        Self { prev, func }
    }
}

impl<Prev, Func, Out> BaseSurface for MapSurface<Prev, Func>
where
    Prev: BaseSurface,
    Func: FnMut(&Context, Prev::ItemOut) -> Out,
{
    type ItemOut = Out;
}

impl<Prev, Func, Out> PullSurface for MapSurface<Prev, Func>
where
    Prev: PullSurface,
    Func: FnMut(&Context, Prev::ItemOut) -> Out,
{
    type InputHandoffs = Prev::InputHandoffs;
    type Build = MapPullBuild<Prev::Build, Func>;

    fn make_parts(self, ctx: &mut Context) -> (Self::InputHandoffs, Self::Build) {
        let (connect, build) = self.prev.make_parts(ctx);
        let build = MapPullBuild::new(build, self.func);
        (connect, build)
    }
}
impl<Prev, Func, Out> AssembleFlowGraph for MapSurface<Prev, Func>
where
    Prev: PullSurface + AssembleFlowGraph,
    Func: FnMut(&Context, Prev::ItemOut) -> Out,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> NodeId {
        let my_id = e.add_node("Map");
        let prev_id = self.prev.insert_dep(e);
        e.add_edge((prev_id, my_id));
        my_id
    }
}

impl<Prev, Func, Out> PushSurface for MapSurface<Prev, Func>
where
    Prev: PushSurface,
    Func: FnMut(&Context, Prev::ItemOut) -> Out,
{
    type Output<Next> = Prev::Output<MapPushSurfaceReversed<Next, Func, Prev::ItemOut>>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>;

    fn push_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    {
        self.prev
            .push_to(MapPushSurfaceReversed::new(next, self.func))
    }
}

pub struct MapPushSurfaceReversed<Next, Func, In>
where
    Next: PushSurfaceReversed,
    Func: FnMut(&Context, In) -> Next::ItemIn,
{
    next: Next,
    func: Func,
    _phantom: PhantomData<fn(In)>,
}
impl<Next, Func, In> MapPushSurfaceReversed<Next, Func, In>
where
    Next: PushSurfaceReversed,
    Func: FnMut(&Context, In) -> Next::ItemIn,
{
    pub fn new(next: Next, func: Func) -> Self {
        Self {
            next,
            func,
            _phantom: PhantomData,
        }
    }
}
impl<Next, Func, In> AssembleFlowGraph for MapPushSurfaceReversed<Next, Func, In>
where
    Next: PushSurfaceReversed + AssembleFlowGraph,
    Func: FnMut(&Context, In) -> Next::ItemIn,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> NodeId {
        let my_id = e.add_node("Map");
        let next_id = self.next.insert_dep(e);
        e.add_edge((my_id, next_id));
        my_id
    }
}

impl<Next, Func, In> PushSurfaceReversed for MapPushSurfaceReversed<Next, Func, In>
where
    Next: PushSurfaceReversed,
    Func: FnMut(&Context, In) -> Next::ItemIn,
{
    type ItemIn = In;

    type OutputHandoffs = Next::OutputHandoffs;
    type Build = MapPushBuild<Next::Build, Func, In>;

    fn make_parts(self, ctx: &mut Context) -> (Self::OutputHandoffs, Self::Build) {
        let (connect, build) = self.next.make_parts(ctx);
        let build = MapPushBuild::new(build, self.func);
        (connect, build)
    }
}
