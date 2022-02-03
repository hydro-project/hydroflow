use super::{PullBuild, PullBuildBase};

use crate::scheduled::handoff::handoff_list::BasePortList;

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
        handoffs: <Self::InputHandoffs as BasePortList<false>>::Ctx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        self.prev.build(handoffs).flatten()
    }
}
