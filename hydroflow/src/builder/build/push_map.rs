use super::{PushBuild, PushBuildBase};

use std::marker::PhantomData;

use crate::compiled::map::Map;
use crate::scheduled::handoff::HandoffList;

pub struct MapPushBuild<Next, Func, In>
where
    Next: PushBuild,
    Func: FnMut(In) -> Next::ItemIn,
{
    next: Next,
    func: Func,
    _phantom: PhantomData<fn(In)>,
}
impl<Next, Func, In> MapPushBuild<Next, Func, In>
where
    Next: PushBuild,
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

#[allow(type_alias_bounds)]
type PushBuildImpl<'slf, 'hof, Next, Func, In>
where
    Next: PushBuild,
= Map<In, Next::ItemIn, impl FnMut(In) -> Next::ItemIn, Next::Build<'slf, 'hof>>;

impl<Next, Func, In> PushBuildBase for MapPushBuild<Next, Func, In>
where
    Next: PushBuild,
    Func: FnMut(In) -> Next::ItemIn,
{
    type ItemIn = In;
    type Build<'slf, 'hof> = PushBuildImpl<'slf, 'hof, Next, Func, In>;
}

impl<Next, Func, In> PushBuild for MapPushBuild<Next, Func, In>
where
    Next: PushBuild,
    Func: FnMut(In) -> Next::ItemIn,
{
    type OutputHandoffs = Next::OutputHandoffs;

    fn build<'slf, 'hof>(
        &'slf mut self,
        handoffs: <Self::OutputHandoffs as HandoffList>::SendCtx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        Map::new(|x| (self.func)(x), self.next.build(handoffs))
    }
}
