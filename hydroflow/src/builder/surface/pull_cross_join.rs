use super::{AssembleFlowGraph, BaseSurface, PullSurface};

use crate::builder::build::pull_cross_join::CrossJoinPullBuild;
use crate::scheduled::context::Context;
use crate::scheduled::flow_graph::NodeId;
use crate::scheduled::handoff::handoff_list::{PortList, PortListSplit};
use crate::scheduled::port::RECV;
use crate::scheduled::type_list::Extend;

pub struct CrossJoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        PortList<RECV> + PortListSplit<RECV, PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
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
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        PortList<RECV> + PortListSplit<RECV, PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    pub fn new(prev_a: PrevA, prev_b: PrevB) -> Self {
        Self { prev_a, prev_b }
    }
}
impl<PrevA, PrevB> AssembleFlowGraph for CrossJoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface + AssembleFlowGraph,
    PrevB: PullSurface + AssembleFlowGraph,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        PortList<RECV> + PortListSplit<RECV, PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> NodeId {
        let my_id = e.add_node("CrossJoin");
        let prev_a_id = self.prev_a.insert_dep(e);
        let prev_b_id = self.prev_b.insert_dep(e);
        e.add_edge((prev_a_id, my_id));
        e.add_edge((prev_b_id, my_id));
        my_id
    }
}

impl<PrevA, PrevB> BaseSurface for CrossJoinPullSurface<PrevA, PrevB>
where
    PrevA: PullSurface,
    PrevB: PullSurface,
    PrevA::ItemOut: 'static + Eq + Clone,
    PrevB::ItemOut: 'static + Eq + Clone,

    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        PortList<RECV> + PortListSplit<RECV, PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
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
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        PortList<RECV> + PortListSplit<RECV, PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    type InputHandoffs = <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended;
    type Build = CrossJoinPullBuild<PrevA::Build, PrevB::Build>;

    fn make_parts(self, ctx: &mut Context) -> (Self::InputHandoffs, Self::Build) {
        let (connect_a, build_a) = self.prev_a.make_parts(ctx);
        let (connect_b, build_b) = self.prev_b.make_parts(ctx);
        let connect = connect_a.extend(connect_b);
        let build = CrossJoinPullBuild::new(build_a, build_b);
        (connect, build)
    }
}
