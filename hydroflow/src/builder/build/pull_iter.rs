use super::PullBuild;

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

impl<I> PullBuild for IterPullBuild<I>
where
    I: 'static + Iterator,
{
    type ItemOut = I::Item;
    type Build<'slf, 'ctx> = &'slf mut I;

    type InputHandoffs = ();

    fn build<'slf, 'ctx>(
        &'slf mut self,
        _context: &'ctx Context,
        _handoffs: <Self::InputHandoffs as PortList<RECV>>::Ctx<'ctx>,
    ) -> Self::Build<'slf, 'ctx> {
        &mut self.it
    }
}
