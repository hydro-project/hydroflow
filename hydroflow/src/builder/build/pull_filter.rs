use super::PullBuild;

use crate::scheduled::{context::Context, handoff::handoff_list::PortList, port::RECV};

pub struct FilterPullBuild<Prev, Func>
where
    Prev: PullBuild,
{
    prev: Prev,
    func: Func,
}
impl<Prev, Func> FilterPullBuild<Prev, Func>
where
    Prev: PullBuild,
    Func: FnMut(&Context, &Prev::ItemOut) -> bool,
{
    pub fn new(prev: Prev, func: Func) -> Self {
        Self { prev, func }
    }
}

#[allow(type_alias_bounds)]
type PullBuildImpl<'slf, 'ctx, Prev, Func>
where
    Prev: PullBuild,
    Func: 'slf,
= std::iter::Filter<Prev::Build<'slf, 'ctx>, impl FnMut(&Prev::ItemOut) -> bool>;

impl<Prev, Func> PullBuild for FilterPullBuild<Prev, Func>
where
    Prev: PullBuild,
    Func: FnMut(&Context, &Prev::ItemOut) -> bool,
{
    type ItemOut = Prev::ItemOut;
    type Build<'slf, 'ctx> = PullBuildImpl<'slf, 'ctx, Prev, Func>
    where
        Self: 'slf;

    type InputHandoffs = Prev::InputHandoffs;

    fn build<'slf, 'ctx>(
        &'slf mut self,
        context: &'ctx Context,
        handoffs: <Self::InputHandoffs as PortList<RECV>>::Ctx<'ctx>,
    ) -> Self::Build<'slf, 'ctx> {
        self.prev
            .build(context, handoffs)
            .filter(|x| (self.func)(context, x))
    }
}
