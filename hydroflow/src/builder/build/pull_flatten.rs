use super::{PullBuild, PullBuildBase};

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

impl<Prev> PullBuildBase for FlattenPullBuild<Prev>
where
    Prev: PullBuild,
    Prev::ItemOut: IntoIterator,
{
    type ItemOut = <Prev::ItemOut as IntoIterator>::Item;
    type Build<'slf, 'hof> = std::iter::Flatten<Prev::Build<'slf, 'hof>>;
}

impl<Prev> PullBuild for FlattenPullBuild<Prev>
where
    Prev: PullBuild,
    Prev::ItemOut: IntoIterator,
{
    type InputHandoffs = Prev::InputHandoffs;

    fn build<'slf, 'hof>(
        &'slf mut self,
        context: &Context<'_>,
        handoffs: <Self::InputHandoffs as PortList<RECV>>::Ctx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        self.prev.build(context, handoffs).flatten()
    }
}
