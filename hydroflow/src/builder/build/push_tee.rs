use super::PushBuild;

use crate::scheduled::context::Context;
use crate::scheduled::handoff::handoff_list::{PortList, PortListSplit};
use crate::scheduled::port::SEND;
use crate::scheduled::type_list::Extend;
use pusherator::tee::Tee;

pub struct TeePushBuild<NextA, NextB>
where
    NextA: PushBuild,
    NextB: PushBuild<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    next_a: NextA,
    next_b: NextB,
}
impl<NextA, NextB> TeePushBuild<NextA, NextB>
where
    NextA: PushBuild,
    NextB: PushBuild<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    pub fn new(next_a: NextA, next_b: NextB) -> Self {
        Self { next_a, next_b }
    }
}

impl<NextA, NextB> PushBuild for TeePushBuild<NextA, NextB>
where
    NextA: PushBuild,
    NextB: PushBuild<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    type ItemIn = NextA::ItemIn;
    type Build<'slf, 'ctx> = Tee<NextA::Build<'slf, 'ctx>, NextB::Build<'slf, 'ctx>>
    where
        Self: 'slf;

    type OutputHandoffs = <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended;

    fn build<'slf, 'ctx>(
        &'slf mut self,
        context: &'ctx Context,
        input: <Self::OutputHandoffs as PortList<SEND>>::Ctx<'ctx>,
    ) -> Self::Build<'slf, 'ctx> {
        let (input_a, input_b) = <Self::OutputHandoffs as PortListSplit<_, _>>::split_ctx(input);
        let build_a = self.next_a.build(context, input_a);
        let build_b = self.next_b.build(context, input_b);
        Tee::new(build_a, build_b)
    }
}
