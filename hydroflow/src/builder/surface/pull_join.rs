use super::{BaseSurface, PullSurface};

use std::hash::Hash;

use crate::builder::build::pull_join::JoinPullBuild;
use crate::scheduled::handoff::handoff_list::{BasePortListSplit, RecvPortList};
use crate::scheduled::type_list::Extend;

pub struct JoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface,
    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    prev_a: PrevA,
    prev_b: PrevB,
}
impl<PrevA, PrevB, Key, ValA, ValB> JoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface<ItemOut = (Key, ValA)>,
    PrevB: PullSurface<ItemOut = (Key, ValB)>,
    Key: 'static + Eq + Hash + Clone,
    ValA: 'static + Eq + Clone,
    ValB: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    pub fn new(prev_a: PrevA, prev_b: PrevB) -> Self {
        Self { prev_a, prev_b }
    }
}

impl<PrevA, PrevB, Key, ValA, ValB> BaseSurface for JoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface<ItemOut = (Key, ValA)>,
    PrevB: PullSurface<ItemOut = (Key, ValB)>,
    Key: 'static + Eq + Hash + Clone,
    ValA: 'static + Eq + Clone,
    ValB: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    type ItemOut = (Key, ValA, ValB);
}

impl<PrevA, PrevB, Key, ValA, ValB> PullSurface for JoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface<ItemOut = (Key, ValA)>,
    PrevB: PullSurface<ItemOut = (Key, ValB)>,
    Key: 'static + Eq + Hash + Clone,
    ValA: 'static + Eq + Clone,
    ValB: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended: RecvPortList
        + BasePortListSplit<PrevA::InputHandoffs, false, Suffix = PrevB::InputHandoffs>,
{
    type InputHandoffs = <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended;

    type Build = JoinPullBuild<PrevA::Build, PrevB::Build, Key, ValA, ValB>;

    fn into_build(self) -> Self::Build {
        JoinPullBuild::new(self.prev_a.into_build(), self.prev_b.into_build())
    }
}
