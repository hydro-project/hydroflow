use super::{PullBuild, PullBuildBase};

use crate::scheduled::handoff::HandoffList;

pub struct FilterMapPullBuild<Prev, Func>
where
    Prev: PullBuild,
{
    prev: Prev,
    func: Func,
}
impl<Prev, Func, Out> FilterMapPullBuild<Prev, Func>
where
    Prev: PullBuild,
    Func: FnMut(Prev::ItemOut) -> Option<Out>,
{
    pub fn new(prev: Prev, func: Func) -> Self {
        Self { prev, func }
    }
}

#[allow(type_alias_bounds)]
type PullBuildImpl<'slf, 'hof, Prev, Func, Out>
where
    Prev: PullBuild,
= std::iter::FilterMap<Prev::Build<'slf, 'hof>, impl FnMut(Prev::ItemOut) -> Option<Out>>;

impl<Prev, Func, Out> PullBuildBase for FilterMapPullBuild<Prev, Func>
where
    Prev: PullBuild,
    Func: FnMut(Prev::ItemOut) -> Option<Out>,
{
    type ItemOut = Out;
    type Build<'slf, 'hof> = PullBuildImpl<'slf, 'hof, Prev, Func, Out>;
}

impl<Prev, Func, Out> PullBuild for FilterMapPullBuild<Prev, Func>
where
    Prev: PullBuild,
    Func: FnMut(Prev::ItemOut) -> Option<Out>,
{
    type InputHandoffs = Prev::InputHandoffs;

    fn build<'slf, 'hof>(
        &'slf mut self,
        handoffs: <Self::InputHandoffs as HandoffList>::RecvCtx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        self.prev.build(handoffs).filter_map(|x| (self.func)(x))
    }
}
