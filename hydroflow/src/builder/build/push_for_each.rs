use super::PushBuild;

use std::marker::PhantomData;

use crate::compiled::for_each::ForEach;
use crate::scheduled::context::Context;
use crate::scheduled::handoff::handoff_list::PortList;
use crate::scheduled::port::SEND;
use crate::tt;

pub struct ForEachPushBuild<Func, In>
where
    Func: FnMut(&Context, In),
{
    func: Func,
    _phantom: PhantomData<fn(In)>,
}
impl<Func, In> ForEachPushBuild<Func, In>
where
    Func: FnMut(&Context, In),
{
    pub fn new(func: Func) -> Self {
        Self {
            func,
            _phantom: PhantomData,
        }
    }
}

#[allow(type_alias_bounds)]
type PushBuildImpl<'slf, 'ctx, Func, In>
where
    Func: 'slf,
= ForEach<In, impl FnMut(In)>;

impl<Func, In> PushBuild for ForEachPushBuild<Func, In>
where
    Func: FnMut(&Context, In),
{
    type ItemIn = In;
    type Build<'slf, 'ctx> = PushBuildImpl<'slf, 'ctx, Func, In>
    where
        Self: 'slf;

    type OutputHandoffs = tt!();

    fn build<'slf, 'ctx>(
        &'slf mut self,
        context: &'ctx Context,
        (): <Self::OutputHandoffs as PortList<SEND>>::Ctx<'ctx>,
    ) -> Self::Build<'slf, 'ctx> {
        ForEach::new(|x| (self.func)(context, x))
    }
}
