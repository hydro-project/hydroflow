use super::PullBuild;

use crate::scheduled::{context::Context, handoff::handoff_list::PortList, port::RECV};

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
    Func: FnMut(&Context, Prev::ItemOut) -> Option<Out>,
{
    pub fn new(prev: Prev, func: Func) -> Self {
        Self { prev, func }
    }
}

#[allow(type_alias_bounds)]
type PullBuildImpl<'slf, 'ctx, Prev, Func, Out>
where
    Prev: PullBuild,
    Func: 'slf + FnMut(&Context, Prev::ItemOut) -> Option<Out>,
= std::iter::FilterMap<Prev::Build<'slf, 'ctx>, impl FnMut(Prev::ItemOut) -> Option<Out>>;

impl<Prev, Func, Out> PullBuild for FilterMapPullBuild<Prev, Func>
where
    Prev: PullBuild,
    Func: FnMut(&Context, Prev::ItemOut) -> Option<Out>,
{
    type ItemOut = Out;
    type Build<'slf, 'ctx> = PullBuildImpl<'slf, 'ctx, Prev, Func, Out>
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
            .filter_map(|x| (self.func)(context, x))
    }
}
