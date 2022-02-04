use super::{PullBuild, PullBuildBase};

use std::hash::Hash;

use crate::compiled::pull::{JoinState, SymmetricHashJoin};
use crate::scheduled::handoff::handoff_list::{PortList, PortListSplit};
use crate::scheduled::port::RECV;
use crate::scheduled::type_list::Extend;

pub struct JoinPullBuild<PrevA, PrevB, Key, ValA, ValB>
where
    PrevA: PullBuild<ItemOut = (Key, ValA)>,
    PrevB: PullBuild<ItemOut = (Key, ValB)>,
    Key: 'static + Eq + Hash + Clone,
    ValA: 'static + Eq + Clone,
    ValB: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        PortList<RECV> + PortListSplit<RECV, PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    prev_a: PrevA,
    prev_b: PrevB,
    state: JoinState<Key, ValA, ValB>,
}
impl<PrevA, PrevB, Key, ValA, ValB> JoinPullBuild<PrevA, PrevB, Key, ValA, ValB>
where
    PrevA: PullBuild<ItemOut = (Key, ValA)>,
    PrevB: PullBuild<ItemOut = (Key, ValB)>,
    Key: 'static + Eq + Hash + Clone,
    ValA: 'static + Eq + Clone,
    ValB: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        PortList<RECV> + PortListSplit<RECV, PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    pub fn new(prev_a: PrevA, prev_b: PrevB) -> Self {
        Self {
            prev_a,
            prev_b,
            state: Default::default(),
        }
    }
}

impl<PrevA, PrevB, Key, ValA, ValB> PullBuildBase for JoinPullBuild<PrevA, PrevB, Key, ValA, ValB>
where
    PrevA: PullBuild<ItemOut = (Key, ValA)>,
    PrevB: PullBuild<ItemOut = (Key, ValB)>,
    Key: 'static + Eq + Hash + Clone,
    ValA: 'static + Eq + Clone,
    ValB: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        PortList<RECV> + PortListSplit<RECV, PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    type ItemOut = (Key, ValA, ValB);
    type Build<'slf, 'hof> = SymmetricHashJoin<
        'slf,
        Key,
        PrevA::Build<'slf, 'hof>,
        ValA,
        PrevB::Build<'slf, 'hof>,
        ValB,
    >;
}

impl<PrevA, PrevB, Key, ValA, ValB> PullBuild for JoinPullBuild<PrevA, PrevB, Key, ValA, ValB>
where
    PrevA: PullBuild<ItemOut = (Key, ValA)>,
    PrevB: PullBuild<ItemOut = (Key, ValB)>,
    Key: 'static + Eq + Hash + Clone,
    ValA: 'static + Eq + Clone,
    ValB: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        PortList<RECV> + PortListSplit<RECV, PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    type InputHandoffs = <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended;

    fn build<'slf, 'hof>(
        &'slf mut self,
        input: <Self::InputHandoffs as PortList<RECV>>::Ctx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        let (input_a, input_b) = <Self::InputHandoffs as PortListSplit<_, _>>::split_ctx(input);
        let iter_a = self.prev_a.build(input_a);
        let iter_b = self.prev_b.build(input_b);
        SymmetricHashJoin::new(iter_a, iter_b, &mut self.state)
    }
}
