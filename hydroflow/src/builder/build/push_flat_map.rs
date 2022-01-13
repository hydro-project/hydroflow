use super::{PushBuild, PushBuildBase};

use std::marker::PhantomData;

use crate::compiled::flat_map::FlatMap;
use crate::scheduled::handoff::HandoffList;

pub struct FlatMapPushBuild<Next, Func, In>
where
    Next: PushBuild,
{
    next: Next,
    func: Func,
    _phantom: PhantomData<fn(In)>,
}
impl<Next, Func, In, Out> FlatMapPushBuild<Next, Func, In>
where
    Next: PushBuild,
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

#[allow(type_alias_bounds)]
type PushBuildImpl<'slf, 'hof, Next, Func, In, Out>
where
    Next: PushBuild,
= FlatMap<In, Out, impl FnMut(In) -> Out, Next::Build<'slf, 'hof>>;

impl<Next, Func, In, Out> PushBuildBase for FlatMapPushBuild<Next, Func, In>
where
    Next: PushBuild,
    Func: FnMut(In) -> Out,
    Out: IntoIterator<Item = Next::ItemIn>,
{
    type ItemIn = In;
    type Build<'slf, 'hof> = PushBuildImpl<'slf, 'hof, Next, Func, In, Out>;
}

impl<Next, Func, In, Out> PushBuild for FlatMapPushBuild<Next, Func, In>
where
    Next: PushBuild,
    Func: FnMut(In) -> Out,
    Out: IntoIterator<Item = Next::ItemIn>,
{
    type OutputHandoffs = Next::OutputHandoffs;

    fn build<'slf, 'hof>(
        &'slf mut self,
        handoffs: <Self::OutputHandoffs as HandoffList>::SendCtx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        FlatMap::new(|x| (self.func)(x), self.next.build(handoffs))
    }
}
