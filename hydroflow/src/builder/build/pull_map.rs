use super::{PullBuild, PullBuildBase};

use crate::scheduled::{handoff::handoff_list::PortList, port::RECV};

pub struct MapPullBuild<Prev, Func>
where
    Prev: PullBuild,
{
    prev: Prev,
    func: Func,
}
impl<Prev, Func, Out> MapPullBuild<Prev, Func>
where
    Prev: PullBuild,
    Func: FnMut(Prev::ItemOut) -> Out,
{
    pub fn new(prev: Prev, func: Func) -> Self {
        Self { prev, func }
    }
}

#[allow(type_alias_bounds)]
type PullBuildImpl<'slf, 'hof, Prev, Func, Out>
where
    Prev: PullBuild,
= std::iter::Map<Prev::Build<'slf, 'hof>, impl FnMut(Prev::ItemOut) -> Out>;

impl<Prev, Func, Out> PullBuildBase for MapPullBuild<Prev, Func>
where
    Prev: PullBuild,
    Func: FnMut(Prev::ItemOut) -> Out,
{
    type ItemOut = Out;
    type Build<'slf, 'hof> = PullBuildImpl<'slf, 'hof, Prev, Func, Out>;
}

impl<Prev, Func, Out> PullBuild for MapPullBuild<Prev, Func>
where
    Prev: PullBuild,
    Func: FnMut(Prev::ItemOut) -> Out,
{
    type InputHandoffs = Prev::InputHandoffs;

    fn build<'slf, 'hof>(
        &'slf mut self,
        handoffs: <Self::InputHandoffs as PortList<RECV>>::Ctx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        self.prev.build(handoffs).map(|x| (self.func)(x))
    }
}
