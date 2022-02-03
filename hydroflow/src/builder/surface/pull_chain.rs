use super::{BaseSurface, PullSurface};

use crate::builder::build::pull_chain::ChainPullBuild;
use crate::scheduled::handoff::handoff_list::{BasePortListSplit, RecvPortList};
use crate::scheduled::type_list::Extend;

pub struct ChainPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface<ItemOut = PrevA::ItemOut>,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    prev_a: PrevA,
    prev_b: PrevB,
}
impl<PrevA, PrevB> ChainPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface<ItemOut = PrevA::ItemOut>,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    pub fn new(prev_a: PrevA, prev_b: PrevB) -> Self {
        Self { prev_a, prev_b }
    }
}

impl<PrevA, PrevB> BaseSurface for ChainPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface<ItemOut = PrevA::ItemOut>,
{
    type ItemOut = PrevA::ItemOut;
}

impl<PrevA, PrevB> PullSurface for ChainPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface<ItemOut = PrevA::ItemOut>,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    type InputHandoffs = <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended;

    type Build = ChainPullBuild<PrevA::Build, PrevB::Build>;

    fn into_build(self) -> Self::Build {
        ChainPullBuild::new(self.prev_a.into_build(), self.prev_b.into_build())
    }
}
