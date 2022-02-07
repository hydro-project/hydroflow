use super::{BaseSurface, PullSurface, PushSurface, PushSurfaceReversed};

use std::marker::PhantomData;

use crate::builder::build::pull_flatten::FlattenPullBuild;
use crate::builder::build::push_flatten::FlattenPushBuild;

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

impl<Prev> PushSurface for FlattenSurface<Prev>
where
    Prev: PushSurface,
    Prev::ItemOut: IntoIterator,
{
    type Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    = Prev::Output<FlattenPushSurfaceReversed<Next, Prev::ItemOut>>;

    fn push_into<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    {
        self.prev.push_into(FlattenPushSurfaceReversed::new(next))
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
