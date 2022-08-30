use super::PushBuild;

use crate::scheduled::context::Context;
use crate::scheduled::handoff::handoff_list::PortList;
use crate::scheduled::port::SEND;
use pusherator::filter::Filter;

pub struct FilterPushBuild<Next, Func>
where
    Next: PushBuild,
    Func: FnMut(&Context, &Next::ItemIn) -> bool,
{
    next: Next,
    func: Func,
}
impl<Next, Func> FilterPushBuild<Next, Func>
where
    Next: PushBuild,
    Func: FnMut(&Context, &Next::ItemIn) -> bool,
{
    pub fn new(next: Next, func: Func) -> Self {
        Self { next, func }
    }
}

#[allow(type_alias_bounds)]
type PushBuildImpl<'slf, 'ctx, Next, Func>
where
    Next: PushBuild,
    Func: 'slf + FnMut(&Context, &Next::ItemIn) -> bool,
= Filter<Next::Build<'slf, 'ctx>, impl FnMut(&Next::ItemIn) -> bool>;

impl<Next, Func> PushBuild for FilterPushBuild<Next, Func>
where
    Next: PushBuild,
    Func: FnMut(&Context, &Next::ItemIn) -> bool,
{
    type ItemIn = Next::ItemIn;
    type Build<'slf, 'ctx> = PushBuildImpl<'slf, 'ctx, Next, Func>
    where
        Self: 'slf;

    type OutputHandoffs = Next::OutputHandoffs;

    fn build<'slf, 'ctx>(
        &'slf mut self,
        context: &'ctx Context,
        handoffs: <Self::OutputHandoffs as PortList<SEND>>::Ctx<'ctx>,
    ) -> Self::Build<'slf, 'ctx> {
        Filter::new(
            |x| (self.func)(context, x),
            self.next.build(context, handoffs),
        )
    }
}
