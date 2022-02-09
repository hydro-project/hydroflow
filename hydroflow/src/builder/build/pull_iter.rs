use super::{PullBuild, PullBuildBase};

use crate::scheduled::{context::Context, handoff::handoff_list::PortList, port::RECV};

pub struct IterPullBuild<I>
where
    I: Iterator,
{
    it: I,
}
impl<I> IterPullBuild<I>
where
    I: Iterator,
{
    pub fn new(it: I) -> Self {
        Self { it }
    }
}

impl<I> PullBuildBase for IterPullBuild<I>
where
    I: 'static + Iterator,
{
    type ItemOut = I::Item;
    type Build<'slf, 'hof> = &'slf mut I;
}

impl<I> PullBuild for IterPullBuild<I>
where
    I: 'static + Iterator,
{
    type InputHandoffs = ();

    fn build<'slf, 'hof>(
        &'slf mut self,
        _context: &Context<'_>,
        _handoffs: <Self::InputHandoffs as PortList<RECV>>::Ctx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        &mut self.it
    }
}
