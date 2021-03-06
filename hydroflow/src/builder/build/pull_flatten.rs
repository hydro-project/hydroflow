use super::PullBuild;

use crate::scheduled::{context::Context, handoff::handoff_list::PortList, port::RECV};

pub struct FlattenPullBuild<Prev>
where
    Prev: PullBuild,
{
    prev: Prev,
}
impl<Prev> FlattenPullBuild<Prev>
where
    Prev: PullBuild,
    Prev::ItemOut: IntoIterator,
{
    pub fn new(prev: Prev) -> Self {
        Self { prev }
    }
}

impl<Prev> PullBuild for FlattenPullBuild<Prev>
where
    Prev: PullBuild,
    Prev::ItemOut: IntoIterator,
{
    type ItemOut = <Prev::ItemOut as IntoIterator>::Item;
    type Build<'slf, 'ctx> = std::iter::Flatten<Prev::Build<'slf, 'ctx>>
    where
        Self: 'slf;

    type InputHandoffs = Prev::InputHandoffs;

    fn build<'slf, 'ctx>(
        &'slf mut self,
        context: &'ctx Context,
        handoffs: <Self::InputHandoffs as PortList<RECV>>::Ctx<'ctx>,
    ) -> Self::Build<'slf, 'ctx> {
        self.prev.build(context, handoffs).flatten()
    }
}
