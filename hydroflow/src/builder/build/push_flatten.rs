use super::PushBuild;

use std::marker::PhantomData;

use crate::scheduled::context::Context;
use crate::scheduled::handoff::handoff_list::PortList;
use crate::scheduled::port::SEND;
use pusherator::flatten::Flatten;

pub struct FlattenPushBuild<Next, In>
where
    Next: PushBuild,
    In: IntoIterator<Item = Next::ItemIn>,
{
    next: Next,
    _phantom: PhantomData<fn(In)>,
}
impl<Next, In> FlattenPushBuild<Next, In>
where
    Next: PushBuild,
    In: IntoIterator<Item = Next::ItemIn>,
{
    pub fn new(next: Next) -> Self {
        Self {
            next,
            _phantom: PhantomData,
        }
    }
}

impl<Next, In> PushBuild for FlattenPushBuild<Next, In>
where
    Next: PushBuild,
    In: IntoIterator<Item = Next::ItemIn>,
{
    type ItemIn = In;
    type Build<'slf, 'ctx> = Flatten<Next::Build<'slf, 'ctx>, In>
    where
        Self: 'slf;

    type OutputHandoffs = Next::OutputHandoffs;

    fn build<'slf, 'ctx>(
        &'slf mut self,
        context: &'ctx Context,
        handoffs: <Self::OutputHandoffs as PortList<SEND>>::Ctx<'ctx>,
    ) -> Self::Build<'slf, 'ctx> {
        Flatten::new(self.next.build(context, handoffs))
    }
}
