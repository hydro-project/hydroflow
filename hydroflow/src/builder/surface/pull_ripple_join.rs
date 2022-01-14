use super::{BaseSurface, PullSurface};

use crate::builder::build::pull_ripple_join::RippleJoinPullBuild;
use crate::builder::connect::BinaryPullConnect;
use crate::scheduled::handoff::{HandoffList, HandoffListSplit};
use crate::scheduled::type_list::Extend;

pub struct RippleJoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,
{
    prev_a: PrevA,
    prev_b: PrevB,
}
impl<PrevA, PrevB> RippleJoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,
{
    pub fn new(prev_a: PrevA, prev_b: PrevB) -> Self {
        Self { prev_a, prev_b }
    }
}

impl<PrevA, PrevB> BaseSurface for RippleJoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,
{
    type ItemOut = (PrevA::ItemOut, PrevB::ItemOut);
}

impl<PrevA, PrevB> PullSurface for RippleJoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        HandoffList + HandoffListSplit<PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    type InputHandoffs = <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended;

    type Connect = BinaryPullConnect<PrevA::Connect, PrevB::Connect>;
    type Build = RippleJoinPullBuild<PrevA::Build, PrevB::Build>;

    fn into_parts(self) -> (Self::Connect, Self::Build) {
        let (connect_a, build_a) = self.prev_a.into_parts();
        let (connect_b, build_b) = self.prev_b.into_parts();
        let connect = BinaryPullConnect::new(connect_a, connect_b);
        let build = RippleJoinPullBuild::new(build_a, build_b);
        (connect, build)
    }
}
