use super::{PullBuild, PullBuildBase};

use crate::scheduled::context::Context;
use crate::scheduled::handoff::handoff_list::{PortList, PortListSplit};
use crate::scheduled::port::RECV;
use crate::scheduled::type_list::Extend;

pub struct ChainPullBuild<PrevA, PrevB>
where
    PrevA: PullBuild,
    PrevB: PullBuild<ItemOut = PrevA::ItemOut>,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        PortList<RECV> + PortListSplit<RECV, PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    prev_a: PrevA,
    prev_b: PrevB,
}

impl<PrevA, PrevB> ChainPullBuild<PrevA, PrevB>
where
    PrevA: PullBuild,
    PrevB: PullBuild<ItemOut = PrevA::ItemOut>,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        PortList<RECV> + PortListSplit<RECV, PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    pub fn new(prev_a: PrevA, prev_b: PrevB) -> Self {
        Self { prev_a, prev_b }
    }
}

impl<PrevA, PrevB> PullBuildBase for ChainPullBuild<PrevA, PrevB>
where
    PrevA: PullBuild,
    PrevB: PullBuild<ItemOut = PrevA::ItemOut>,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        PortList<RECV> + PortListSplit<RECV, PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    type ItemOut = PrevA::ItemOut;
    type Build<'slf, 'ctx> = std::iter::Chain<PrevA::Build<'slf, 'ctx>, PrevB::Build<'slf, 'ctx>>;
}

impl<PrevA, PrevB> PullBuild for ChainPullBuild<PrevA, PrevB>
where
    PrevA: PullBuild,
    PrevB: PullBuild<ItemOut = PrevA::ItemOut>,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        PortList<RECV> + PortListSplit<RECV, PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    type InputHandoffs = <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended;

    fn build<'slf, 'ctx>(
        &'slf mut self,
        context: &'ctx Context<'ctx>,
        input: <Self::InputHandoffs as PortList<RECV>>::Ctx<'ctx>,
    ) -> Self::Build<'slf, 'ctx> {
        let (input_a, input_b) = <Self::InputHandoffs as PortListSplit<_, _>>::split_ctx(input);
        let iter_a = self.prev_a.build(context, input_a);
        let iter_b = self.prev_b.build(context, input_b);
        iter_a.chain(iter_b)
    }
}
