use super::{PullBuild, PullBuildBase};

use crate::scheduled::{context::Context, handoff::handoff_list::PortList, port::RECV};

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
type PullBuildImpl<'slf, 'ctx, Prev, Func, Out>
where
    Prev: PullBuild,
= std::iter::Map<Prev::Build<'slf, 'ctx>, impl FnMut(Prev::ItemOut) -> Out>;

impl<Prev, Func, Out> PullBuildBase for MapPullBuild<Prev, Func>
where
    Prev: PullBuild,
    Func: FnMut(Prev::ItemOut) -> Out,
{
    type ItemOut = Out;
    type Build<'slf, 'ctx> = PullBuildImpl<'slf, 'ctx, Prev, Func, Out>;
}

impl<Prev, Func, Out> PullBuild for MapPullBuild<Prev, Func>
where
    Prev: PullBuild,
    Func: FnMut(Prev::ItemOut) -> Out,
{
    type InputHandoffs = Prev::InputHandoffs;

    fn build<'slf, 'ctx>(
        &'slf mut self,
        context: &'ctx Context<'ctx>,
        handoffs: <Self::InputHandoffs as PortList<RECV>>::Ctx<'ctx>,
    ) -> Self::Build<'slf, 'ctx> {
        self.prev.build(context, handoffs).map(|x| (self.func)(x))
    }
}
