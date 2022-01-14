use super::{PullBuild, PullBuildBase};

use crate::compiled::pull::{RippleJoin, RippleJoinState};
use crate::scheduled::handoff::{HandoffList, HandoffListSplit};
use crate::scheduled::type_list::Extend;

pub struct RippleJoinPullBuild<PrevA, PrevB>
where
    PrevA: PullBuild,
    PrevB: PullBuild,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,
{
    prev_a: PrevA,
    prev_b: PrevB,
    state: RippleJoinState<PrevA::ItemOut, PrevB::ItemOut>,
}
impl<PrevA, PrevB> RippleJoinPullBuild<PrevA, PrevB>
where
    PrevA: PullBuild,
    PrevB: PullBuild,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,
{
    pub fn new(prev_a: PrevA, prev_b: PrevB) -> Self {
        Self {
            prev_a,
            prev_b,
            state: Default::default(),
        }
    }
}

impl<PrevA, PrevB> PullBuildBase for RippleJoinPullBuild<PrevA, PrevB>
where
    PrevA: PullBuild,
    PrevB: PullBuild,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,
{
    type ItemOut = (PrevA::ItemOut, PrevB::ItemOut);
    type Build<'slf, 'hof> = RippleJoin<
        'slf,
        PrevA::Build<'slf, 'hof>,
        PrevA::ItemOut,
        PrevB::Build<'slf, 'hof>,
        PrevB::ItemOut,
    >;
}

impl<PrevA, PrevB> PullBuild for RippleJoinPullBuild<PrevA, PrevB>
where
    PrevA: PullBuild,
    PrevB: PullBuild,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        HandoffList + HandoffListSplit<PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    type InputHandoffs = <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended;

    fn build<'slf, 'hof>(
        &'slf mut self,
        input: <Self::InputHandoffs as HandoffList>::RecvCtx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        let (input_a, input_b) =
            <Self::InputHandoffs as HandoffListSplit<_>>::split_recv_ctx(input);
        let iter_a = self.prev_a.build(input_a);
        let iter_b = self.prev_b.build(input_b);
        RippleJoin::new(iter_a, iter_b, &mut self.state)
    }
}
