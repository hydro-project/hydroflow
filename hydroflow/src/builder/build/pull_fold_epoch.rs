use super::{PullBuild, PullBuildBase};

use crate::scheduled::{context::Context, handoff::handoff_list::PortList, port::RECV};

pub struct FoldEpochPullBuild<Prev, Init, Func>
where
    Prev: PullBuild,
{
    prev: Prev,
    init: Init,
    func: Func,
}
impl<Prev, Init, Func, Out> FoldEpochPullBuild<Prev, Init, Func>
where
    Prev: PullBuild,
    Init: FnMut(&Context<'_>) -> Out,
    Func: FnMut(&Context<'_>, Out, Prev::ItemOut) -> Out,
{
    pub fn new(prev: Prev, init: Init, func: Func) -> Self {
        Self { prev, init, func }
    }
}

impl<Prev, Init, Func, Out> PullBuildBase for FoldEpochPullBuild<Prev, Init, Func>
where
    Prev: PullBuild,
    Init: FnMut(&Context<'_>) -> Out,
    Func: FnMut(&Context<'_>, Out, Prev::ItemOut) -> Out,
{
    type ItemOut = Out;
    type Build<'slf, 'ctx> = std::iter::Once<Out>;
}

impl<Prev, Init, Func, Out> PullBuild for FoldEpochPullBuild<Prev, Init, Func>
where
    Prev: PullBuild,
    Init: FnMut(&Context<'_>) -> Out,
    Func: FnMut(&Context<'_>, Out, Prev::ItemOut) -> Out,
{
    type InputHandoffs = Prev::InputHandoffs;

    fn build<'slf, 'ctx>(
        &'slf mut self,
        context: &'ctx Context<'ctx>,
        handoffs: <Self::InputHandoffs as PortList<RECV>>::Ctx<'ctx>,
    ) -> Self::Build<'slf, 'ctx> {
        std::iter::once(
            self.prev
                .build(context, handoffs)
                .fold((self.init)(context), |acc, x| (self.func)(context, acc, x)),
        )
    }
}
