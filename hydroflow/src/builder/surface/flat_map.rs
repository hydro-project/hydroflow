use super::{BaseSurface, PullSurface, PushSurface, PushSurfaceReversed};

use std::marker::PhantomData;

use crate::builder::build::pull_flat_map::FlatMapPullBuild;
use crate::builder::build::push_flat_map::FlatMapPushBuild;

pub struct FlatMapSurface<Prev, Func>
where
    Prev: BaseSurface,
{
    prev: Prev,
    func: Func,
}
impl<Prev, Func, Out> FlatMapSurface<Prev, Func>
where
    Prev: BaseSurface,
    Func: FnMut(Prev::ItemOut) -> Out,
    Out: IntoIterator,
{
    pub fn new(prev: Prev, func: Func) -> Self {
        Self { prev, func }
    }
}

impl<Prev, Func, Out> BaseSurface for FlatMapSurface<Prev, Func>
where
    Prev: BaseSurface,
    Func: FnMut(Prev::ItemOut) -> Out,
    Out: IntoIterator,
{
    type ItemOut = Out::Item;
}

impl<Prev, Func, Out> PullSurface for FlatMapSurface<Prev, Func>
where
    Prev: PullSurface,
    Func: FnMut(Prev::ItemOut) -> Out,
    Out: IntoIterator,
{
    type InputHandoffs = Prev::InputHandoffs;

    type Connect = Prev::Connect;
    type Build = FlatMapPullBuild<Prev::Build, Func>;

    fn into_parts(self) -> (Self::Connect, Self::Build) {
        let (connect, build) = self.prev.into_parts();
        let build = FlatMapPullBuild::new(build, self.func);
        (connect, build)
    }
}

impl<Prev, Func, Out> PushSurface for FlatMapSurface<Prev, Func>
where
    Prev: PushSurface,
    Func: FnMut(Prev::ItemOut) -> Out,
    Out: IntoIterator,
{
    type Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    = Prev::Output<FlatMapPushSurfaceReversed<Next, Func, Prev::ItemOut>>;

    fn reverse<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    {
        self.prev
            .reverse(FlatMapPushSurfaceReversed::new(next, self.func))
    }
}

pub struct FlatMapPushSurfaceReversed<Next, Func, In>
where
    Next: PushSurfaceReversed,
{
    next: Next,
    func: Func,
    _phantom: PhantomData<fn(In)>,
}
impl<Next, Func, In, Out> FlatMapPushSurfaceReversed<Next, Func, In>
where
    Next: PushSurfaceReversed,
    Func: FnMut(In) -> Out,
    Out: IntoIterator<Item = Next::ItemIn>,
{
    pub fn new(next: Next, func: Func) -> Self {
        Self {
            next,
            func,
            _phantom: PhantomData,
        }
    }
}

impl<Next, Func, In, Out> PushSurfaceReversed for FlatMapPushSurfaceReversed<Next, Func, In>
where
    Next: PushSurfaceReversed,
    Func: FnMut(In) -> Out,
    Out: IntoIterator<Item = Next::ItemIn>,
{
    type OutputHandoffs = Next::OutputHandoffs;

    type ItemIn = In;

    type Connect = Next::Connect;
    type Build = FlatMapPushBuild<Next::Build, Func, In>;

    fn into_parts(self) -> (Self::Connect, Self::Build) {
        let (connect, build) = self.next.into_parts();
        let build = FlatMapPushBuild::new(build, self.func);
        (connect, build)
    }
}
