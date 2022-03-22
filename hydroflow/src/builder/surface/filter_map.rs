use super::{
    BaseSurface, PullSurface, PushSurface, PushSurfaceReversed, TrackPullDependencies,
    TrackPushDependencies,
};

use std::marker::PhantomData;

use crate::builder::build::pull_filter_map::FilterMapPullBuild;
use crate::builder::build::push_filter_map::FilterMapPushBuild;
use crate::scheduled::context::Context;

pub struct FilterMapSurface<Prev, Func>
where
    Prev: BaseSurface,
{
    prev: Prev,
    func: Func,
}
impl<Prev, Func, Out> FilterMapSurface<Prev, Func>
where
    Prev: BaseSurface,
    Func: FnMut(&Context<'_>, Prev::ItemOut) -> Option<Out>,
{
    pub fn new(prev: Prev, func: Func) -> Self {
        Self { prev, func }
    }
}

impl<Prev, Func, Out> BaseSurface for FilterMapSurface<Prev, Func>
where
    Prev: BaseSurface,
    Func: FnMut(&Context<'_>, Prev::ItemOut) -> Option<Out>,
{
    type ItemOut = Out;
}

impl<Prev, Func, Out> PullSurface for FilterMapSurface<Prev, Func>
where
    Prev: PullSurface,
    Func: FnMut(&Context<'_>, Prev::ItemOut) -> Option<Out>,
{
    type InputHandoffs = Prev::InputHandoffs;
    type Build = FilterMapPullBuild<Prev::Build, Func>;

    fn into_parts(self) -> (Self::InputHandoffs, Self::Build) {
        let (connect, build) = self.prev.into_parts();
        let build = FilterMapPullBuild::new(build, self.func);
        (connect, build)
    }
}
impl<Prev, Func, Out> TrackPullDependencies for FilterMapSurface<Prev, Func>
where
    Prev: PullSurface + TrackPullDependencies,
    Func: FnMut(&Context<'_>, Prev::ItemOut) -> Option<Out>,
{
    fn insert_dep(&self, e: &mut super::DirectedEdgeSet) -> u16 {
        let my_id = e.add_node("FilterMap".to_string());
        let prev_id = self.prev.insert_dep(e);
        e.add_edge((prev_id, my_id));
        my_id
    }
}

impl<Prev, Func, Out> PushSurface for FilterMapSurface<Prev, Func>
where
    Prev: PushSurface,
    Func: FnMut(&Context<'_>, Prev::ItemOut) -> Option<Out>,
{
    type Output<Next> = Prev::Output<FilterMapPushSurfaceReversed<Next, Func, Prev::ItemOut>>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>;

    fn push_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    {
        self.prev
            .push_to(FilterMapPushSurfaceReversed::new(next, self.func))
    }
}
impl<Prev, Func, Out> TrackPushDependencies for FilterMapSurface<Prev, Func>
where
    Prev: PushSurface + TrackPushDependencies,
    Func: FnMut(&Context<'_>, &Prev::ItemOut) -> Option<Out>,
{
    fn insert_dep(&self, e: &mut super::DirectedEdgeSet) -> u16 {
        let my_id = e.add_node("FilterMap".to_string());
        let prev_id = self.prev.insert_dep(e);
        e.add_edge((prev_id, my_id));
        my_id
    }
}

pub struct FilterMapPushSurfaceReversed<Next, Func, In>
where
    Next: PushSurfaceReversed,
{
    next: Next,
    func: Func,
    _phantom: PhantomData<fn(In)>,
}
impl<Next, Func, In> FilterMapPushSurfaceReversed<Next, Func, In>
where
    Next: PushSurfaceReversed,
    Func: FnMut(&Context<'_>, In) -> Option<Next::ItemIn>,
{
    pub fn new(next: Next, func: Func) -> Self {
        Self {
            next,
            func,
            _phantom: PhantomData,
        }
    }
}

impl<Next, Func, In> PushSurfaceReversed for FilterMapPushSurfaceReversed<Next, Func, In>
where
    Next: PushSurfaceReversed,
    Func: FnMut(&Context<'_>, In) -> Option<Next::ItemIn>,
{
    type ItemIn = In;

    type OutputHandoffs = Next::OutputHandoffs;
    type Build = FilterMapPushBuild<Next::Build, Func, In>;

    fn into_parts(self) -> (Self::OutputHandoffs, Self::Build) {
        let (connect, build) = self.next.into_parts();
        let build = FilterMapPushBuild::new(build, self.func);
        (connect, build)
    }
}
