use super::PullBuild;

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
    Init: FnMut(&Context) -> Out,
    Func: FnMut(&Context, Out, Prev::ItemOut) -> Out,
{
    pub fn new(prev: Prev, init: Init, func: Func) -> Self {
        Self { prev, init, func }
    }
}

#[allow(type_alias_bounds)]
type PullBuildImpl<'slf, 'ctx, Prev, Init, Func, Out>
where
    Prev: 'slf + PullBuild,
    Init: 'slf + FnMut(&Context) -> Out,
    Func: 'slf + FnMut(&Context, Out, Prev::ItemOut) -> Out,
= std::iter::OnceWith<impl FnOnce() -> Out>;

impl<Prev, Init, Func, Out> PullBuild for FoldEpochPullBuild<Prev, Init, Func>
where
    Prev: PullBuild,
    Init: FnMut(&Context) -> Out,
    Func: FnMut(&Context, Out, Prev::ItemOut) -> Out,
{
    type ItemOut = Out;
    type Build<'slf, 'ctx> = PullBuildImpl<'slf, 'ctx, Prev, Init, Func, Out>
    where
        Self: 'slf;

    type InputHandoffs = Prev::InputHandoffs;

    fn build<'slf, 'ctx>(
        &'slf mut self,
        context: &'ctx Context,
        handoffs: <Self::InputHandoffs as PortList<RECV>>::Ctx<'ctx>,
    ) -> Self::Build<'slf, 'ctx> {
        std::iter::once_with(move || {
            self.prev
                .build(context, handoffs)
                .fold((self.init)(context), |acc, x| (self.func)(context, acc, x))
        })
    }
}
