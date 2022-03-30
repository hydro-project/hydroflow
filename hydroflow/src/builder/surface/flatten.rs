use super::{AssembleFlowGraph, BaseSurface, PullSurface, PushSurface, PushSurfaceReversed};

use std::marker::PhantomData;

use crate::builder::build::pull_flatten::FlattenPullBuild;
use crate::builder::build::push_flatten::FlattenPushBuild;
use crate::scheduled::flow_graph::NodeId;

pub struct FlattenSurface<Prev>
where
    Prev: BaseSurface,
    Prev::ItemOut: IntoIterator,
{
    prev: Prev,
}
impl<Prev> FlattenSurface<Prev>
where
    Prev: BaseSurface,
    Prev::ItemOut: IntoIterator,
{
    pub fn new(prev: Prev) -> Self {
        Self { prev }
    }
}

impl<Prev> BaseSurface for FlattenSurface<Prev>
where
    Prev: BaseSurface,
    Prev::ItemOut: IntoIterator,
{
    type ItemOut = <Prev::ItemOut as IntoIterator>::Item;
}

impl<Prev> PullSurface for FlattenSurface<Prev>
where
    Prev: PullSurface,
    Prev::ItemOut: IntoIterator,
{
    type InputHandoffs = Prev::InputHandoffs;
    type Build = FlattenPullBuild<Prev::Build>;

    fn into_parts(self) -> (Self::InputHandoffs, Self::Build) {
        let (connect, build) = self.prev.into_parts();
        let build = FlattenPullBuild::new(build);
        (connect, build)
    }
}
impl<Prev> AssembleFlowGraph for FlattenSurface<Prev>
where
    Prev: PullSurface + AssembleFlowGraph,
    Prev::ItemOut: IntoIterator,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> NodeId {
        let my_id = e.add_node("Flatten");
        let prev_id = self.prev.insert_dep(e);
        e.add_edge((prev_id, my_id));
        my_id
    }
}

impl<Prev> PushSurface for FlattenSurface<Prev>
where
    Prev: PushSurface,
    Prev::ItemOut: IntoIterator,
{
    type Output<Next> = Prev::Output<FlattenPushSurfaceReversed<Next, Prev::ItemOut>>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>;

    fn push_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    {
        self.prev.push_to(FlattenPushSurfaceReversed::new(next))
    }
}

pub struct FlattenPushSurfaceReversed<Next, In>
where
    Next: PushSurfaceReversed,
    In: IntoIterator<Item = Next::ItemIn>,
{
    next: Next,
    _phantom: PhantomData<fn(In)>,
}
impl<Next, In> FlattenPushSurfaceReversed<Next, In>
where
    Next: PushSurfaceReversed,
    In: IntoIterator<Item = Next::ItemIn>,
{
    pub fn new(next: Next) -> Self {
        Self {
            next,
            _phantom: PhantomData,
        }
    }
}
impl<Next, In> AssembleFlowGraph for FlattenPushSurfaceReversed<Next, In>
where
    Next: PushSurfaceReversed + AssembleFlowGraph,
    In: IntoIterator<Item = Next::ItemIn>,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> NodeId {
        let my_id = e.add_node("Flatten");
        let next_id = self.next.insert_dep(e);
        e.add_edge((my_id, next_id));
        my_id
    }
}

impl<Next, In> PushSurfaceReversed for FlattenPushSurfaceReversed<Next, In>
where
    Next: PushSurfaceReversed,
    In: IntoIterator<Item = Next::ItemIn>,
{
    type ItemIn = In;

    type OutputHandoffs = Next::OutputHandoffs;
    type Build = FlattenPushBuild<Next::Build, In>;

    fn into_parts(self) -> (Self::OutputHandoffs, Self::Build) {
        let (connect, build) = self.next.into_parts();
        let build = FlattenPushBuild::new(build);
        (connect, build)
    }
}
