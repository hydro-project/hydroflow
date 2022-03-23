use super::{AssembleFlowGraph, PushSurfaceReversed};

use crate::builder::build::push_tee::TeePushBuild;
use crate::scheduled::handoff::handoff_list::{PortList, PortListSplit};
use crate::scheduled::port::SEND;
use crate::scheduled::type_list::Extend;

pub struct TeePushSurfaceReversed<NextA, NextB>
where
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    next_a: NextA,
    next_b: NextB,
}
impl<NextA, NextB> TeePushSurfaceReversed<NextA, NextB>
where
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    pub fn new(next_a: NextA, next_b: NextB) -> Self {
        Self { next_a, next_b }
    }
}
impl<NextA, NextB> AssembleFlowGraph for TeePushSurfaceReversed<NextA, NextB>
where
    NextA: PushSurfaceReversed + AssembleFlowGraph,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn> + AssembleFlowGraph,
    NextA::ItemIn: Clone,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> usize {
        let my_id = e.add_node("Tee".to_string());
        let next_a_id = self.next_a.insert_dep(e);
        let next_b_id = self.next_b.insert_dep(e);
        e.add_edge((my_id, next_a_id));
        e.add_edge((my_id, next_b_id));
        my_id
    }
}

impl<NextA, NextB> PushSurfaceReversed for TeePushSurfaceReversed<NextA, NextB>
where
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    type ItemIn = NextA::ItemIn;

    type OutputHandoffs = <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended;
    type Build = TeePushBuild<NextA::Build, NextB::Build>;

    fn into_parts(self) -> (Self::OutputHandoffs, Self::Build) {
        let (connect_a, build_a) = self.next_a.into_parts();
        let (connect_b, build_b) = self.next_b.into_parts();
        let connect = connect_a.extend(connect_b);
        let build = TeePushBuild::new(build_a, build_b);
        (connect, build)
    }
}
