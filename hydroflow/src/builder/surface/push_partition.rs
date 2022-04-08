use super::{AssembleFlowGraph, PushSurfaceReversed};

use crate::builder::build::push_partition::PartitionPushBuild;
use crate::scheduled::context::Context;
use crate::scheduled::flow_graph::NodeId;
use crate::scheduled::handoff::handoff_list::{PortList, PortListSplit};
use crate::scheduled::port::SEND;
use crate::scheduled::type_list::Extend;

pub struct PartitionPushSurfaceReversed<NextA, NextB, Func>
where
    Func: FnMut(&Context, &NextA::ItemIn) -> bool,
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    func: Func,
    next_a: NextA,
    next_b: NextB,
}
impl<NextA, NextB, Func> PartitionPushSurfaceReversed<NextA, NextB, Func>
where
    Func: FnMut(&Context, &NextA::ItemIn) -> bool,
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    pub fn new(func: Func, next_a: NextA, next_b: NextB) -> Self {
        Self {
            func,
            next_a,
            next_b,
        }
    }
}
impl<NextA, NextB, Func> AssembleFlowGraph for PartitionPushSurfaceReversed<NextA, NextB, Func>
where
    Func: FnMut(&Context, &NextA::ItemIn) -> bool,
    NextA: PushSurfaceReversed + AssembleFlowGraph,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn> + AssembleFlowGraph,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> NodeId {
        let my_id = e.add_node("Partition");
        let next_a_id = self.next_a.insert_dep(e);
        let next_b_id = self.next_b.insert_dep(e);
        e.add_edge((my_id, next_a_id));
        e.add_edge((my_id, next_b_id));
        my_id
    }
}

impl<NextA, NextB, Func> PushSurfaceReversed for PartitionPushSurfaceReversed<NextA, NextB, Func>
where
    Func: FnMut(&Context, &NextA::ItemIn) -> bool,
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    type ItemIn = NextA::ItemIn;

    type OutputHandoffs = <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended;
    type Build = PartitionPushBuild<NextA::Build, NextB::Build, Func>;

    fn into_parts(self, ctx: &mut Context) -> (Self::OutputHandoffs, Self::Build) {
        let (connect_a, build_a) = self.next_a.into_parts(ctx);
        let (connect_b, build_b) = self.next_b.into_parts(ctx);
        let connect = connect_a.extend(connect_b);
        let build = PartitionPushBuild::new(self.func, build_a, build_b);
        (connect, build)
    }
}
