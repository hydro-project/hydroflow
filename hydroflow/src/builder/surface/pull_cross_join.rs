use super::{BaseSurface, PullSurface};

use crate::builder::build::pull_cross_join::CrossJoinPullBuild;
use crate::scheduled::handoff::handoff_list::{BasePortListSplit, RecvPortList};
use crate::scheduled::type_list::Extend;

pub struct CrossJoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    prev_a: PrevA,
    prev_b: PrevB,
}
impl<PrevA, PrevB> CrossJoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    pub fn new(prev_a: PrevA, prev_b: PrevB) -> Self {
        Self { prev_a, prev_b }
    }
}

impl<PrevA, PrevB> BaseSurface for CrossJoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    type ItemOut = (PrevA::ItemOut, PrevB::ItemOut);
}

impl<PrevA, PrevB> PullSurface for CrossJoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    type InputHandoffs = <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended;
    type Build = CrossJoinPullBuild<PrevA::Build, PrevB::Build>;

    fn into_parts(self) -> (Self::InputHandoffs, Self::Build) {
        let (connect_a, build_a) = self.prev_a.into_parts();
        let (connect_b, build_b) = self.prev_b.into_parts();
        let connect = connect_a.extend(connect_b);
        let build = CrossJoinPullBuild::new(build_a, build_b);
        (connect, build)
    }
}
