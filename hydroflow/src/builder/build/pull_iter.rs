use super::{PullBuild, PullBuildBase};

use crate::scheduled::handoff::HandoffList;

pub struct IterPullBuild<I, T>
where
    I: Iterator<Item = T>,
{
    it: I,
}
impl<I, T> IterPullBuild<I, T>
where
    I: Iterator<Item = T>,
{
    pub fn new(it: I) -> Self {
        Self { it }
    }
}

impl<I, T> PullBuildBase for IterPullBuild<I, T>
where
    I: 'static + Iterator<Item = T>,
{
    type ItemOut = T;
    type Build<'slf, 'hof> = &'slf mut I;
}

impl<I, T> PullBuild for IterPullBuild<I, T>
where
    I: 'static + Iterator<Item = T>,
{
    type InputHandoffs = ();

    fn build<'slf, 'hof>(
        &'slf mut self,
        _handoffs: <Self::InputHandoffs as HandoffList>::RecvCtx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        &mut self.it
    }
}
