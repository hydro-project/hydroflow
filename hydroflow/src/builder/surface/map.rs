use super::{BaseSurface, PullSurface, PushSurface, PushSurfaceReversed};

use std::marker::PhantomData;

use crate::builder::build::pull_map::MapPullBuild;
use crate::builder::build::push_map::MapPushBuild;

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
    Func: FnMut(Prev::ItemOut) -> Out,
{
    pub fn new(prev: Prev, func: Func) -> Self {
        Self { prev, func }
    }
}

impl<Prev, Func, Out> BaseSurface for MapSurface<Prev, Func>
where
    Prev: BaseSurface,
    Func: FnMut(Prev::ItemOut) -> Out,
{
    type ItemOut = Out;
}

impl<Prev, Func, Out> PullSurface for MapSurface<Prev, Func>
where
    Prev: PullSurface,
    Func: FnMut(Prev::ItemOut) -> Out,
{
    type InputHandoffs = Prev::InputHandoffs;
    type Build = MapPullBuild<Prev::Build, Func>;

    fn into_parts(self) -> (Self::InputHandoffs, Self::Build) {
        let (connect, build) = self.prev.into_parts();
        let build = MapPullBuild::new(build, self.func);
        (connect, build)
    }
}

impl<Prev, Func, Out> PushSurface for MapSurface<Prev, Func>
where
    Prev: PushSurface,
    Func: FnMut(Prev::ItemOut) -> Out,
{
    type Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    = Prev::Output<MapPushSurfaceReversed<Next, Func, Prev::ItemOut>>;

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
{
    next: Next,
    func: Func,
    _phantom: PhantomData<fn(In)>,
}
impl<Next, Func, In> MapPushSurfaceReversed<Next, Func, In>
where
    Next: PushSurfaceReversed,
    Func: FnMut(In) -> Next::ItemIn,
{
    pub fn new(next: Next, func: Func) -> Self {
        Self {
            next,
            func,
            _phantom: PhantomData,
        }
    }
}

impl<Next, Func, In> PushSurfaceReversed for MapPushSurfaceReversed<Next, Func, In>
where
    Next: PushSurfaceReversed,
    Func: FnMut(In) -> Next::ItemIn,
{
    type ItemIn = In;

    type OutputHandoffs = Next::OutputHandoffs;
    type Build = MapPushBuild<Next::Build, Func, In>;

    fn into_parts(self) -> (Self::OutputHandoffs, Self::Build) {
        let (connect, build) = self.next.into_parts();
        let build = MapPushBuild::new(build, self.func);
        (connect, build)
    }
}
