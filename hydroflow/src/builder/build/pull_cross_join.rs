use super::{PullBuild, PullBuildBase};

use crate::compiled::pull::{CrossJoin, CrossJoinState};
use crate::scheduled::handoff::handoff_list::{BasePortList, BasePortListSplit, RecvPortList};
use crate::scheduled::type_list::Extend;

pub struct CrossJoinPullBuild<PrevA, PrevB>
where
    PrevA: PullBuild,
    PrevB: PullBuild,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    prev_a: PrevA,
    prev_b: PrevB,
    state: CrossJoinState<PrevA::ItemOut, PrevB::ItemOut>,
}
impl<PrevA, PrevB> CrossJoinPullBuild<PrevA, PrevB>
where
    PrevA: PullBuild,
    PrevB: PullBuild,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    pub fn new(prev_a: PrevA, prev_b: PrevB) -> Self {
        Self {
            prev_a,
            prev_b,
            state: Default::default(),
        }
    }
}

impl<PrevA, PrevB> PullBuildBase for CrossJoinPullBuild<PrevA, PrevB>
where
    PrevA: PullBuild,
    PrevB: PullBuild,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    type ItemOut = (PrevA::ItemOut, PrevB::ItemOut);
    type Build<'slf, 'hof> = CrossJoin<
        'slf,
        PrevA::Build<'slf, 'hof>,
        PrevA::ItemOut,
        PrevB::Build<'slf, 'hof>,
        PrevB::ItemOut,
    >;
}

impl<PrevA, PrevB> PullBuild for CrossJoinPullBuild<PrevA, PrevB>
where
    PrevA: PullBuild,
    PrevB: PullBuild,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    type InputHandoffs = <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended;

    fn build<'slf, 'hof>(
        &'slf mut self,
        input: <Self::InputHandoffs as BasePortList<false>>::Ctx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        let (input_a, input_b) =
            <Self::InputHandoffs as BasePortListSplit<_, false>>::split_ctx(input);
        let iter_a = self.prev_a.build(input_a);
        let iter_b = self.prev_b.build(input_b);
        CrossJoin::new(iter_a, iter_b, &mut self.state)
    }
}
