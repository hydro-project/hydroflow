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

    fn into_build(self) -> Self::Build {
        MapPullBuild::new(self.prev.into_build(), self.func)
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

    fn reverse<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    {
        self.prev
            .reverse(MapPushSurfaceReversed::new(next, self.func))
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
    type OutputHandoffs = Next::OutputHandoffs;

    type ItemIn = In;

    type Build = MapPushBuild<Next::Build, Func, In>;

    fn into_build(self) -> Self::Build {
        MapPushBuild::new(self.prev.into_build(), self.func)
    }
}
