use super::{AssembleFlowGraph, BaseSurface, PullSurface};

use std::marker::PhantomData;

use crate::builder::build::pull_batch::BatchPullBuild;
use crate::lang::lattice::{LatticeRepr, Merge};
use crate::scheduled::context::Context;
use crate::scheduled::flow_graph::FlowNodeId;
use crate::scheduled::handoff::handoff_list::{PortList, PortListSplit};
use crate::scheduled::port::RECV;
use crate::scheduled::type_list::Extend;

pub struct BatchPullSurface<PrevBuf, PrevStream, L, Update, Tick>
where
    PrevBuf: PullSurface,
    PrevStream: PullSurface,
    Update: LatticeRepr,
    L: LatticeRepr + Merge<Update>,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    prev_a: PrevBuf,
    prev_b: PrevStream,
    _marker: PhantomData<(L, Update, Tick)>,
}
impl<PrevBuf, PrevStream, L, Update, Tick> BatchPullSurface<PrevBuf, PrevStream, L, Update, Tick>
where
    PrevBuf: PullSurface<ItemOut = Update::Repr>,
    PrevStream: PullSurface<ItemOut = Tick>,
    Update: 'static + LatticeRepr,
    L: 'static + LatticeRepr + Merge<Update>,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    pub fn new(prev_a: PrevBuf, prev_b: PrevStream) -> Self {
        Self {
            prev_a,
            prev_b,
            _marker: PhantomData,
        }
    }
}
impl<PrevBuf, PrevStream, L, Update, Tick> AssembleFlowGraph
    for BatchPullSurface<PrevBuf, PrevStream, L, Update, Tick>
where
    PrevBuf: PullSurface<ItemOut = Update::Repr> + AssembleFlowGraph,
    PrevStream: PullSurface<ItemOut = Tick> + AssembleFlowGraph,
    Update: 'static + LatticeRepr,
    L: 'static + LatticeRepr + Merge<Update>,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> FlowNodeId {
        let my_id = e.add_node("Batch");
        let prev_a_id = self.prev_a.insert_dep(e);
        let prev_b_id = self.prev_b.insert_dep(e);
        e.add_edge((prev_a_id, my_id));
        e.add_edge((prev_b_id, my_id));
        my_id
    }
}

impl<PrevBuf, PrevStream, L, Update, Tick> BaseSurface
    for BatchPullSurface<PrevBuf, PrevStream, L, Update, Tick>
where
    PrevBuf: PullSurface<ItemOut = Update::Repr>,
    PrevStream: PullSurface<ItemOut = Tick>,
    L: 'static + LatticeRepr + Merge<Update>,
    Update: 'static + LatticeRepr,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    type ItemOut = (Tick, L::Repr);
}

impl<PrevBuf, PrevStream, L, Update, Tick> PullSurface
    for BatchPullSurface<PrevBuf, PrevStream, L, Update, Tick>
where
    PrevBuf: PullSurface<ItemOut = Update::Repr>,
    PrevStream: PullSurface<ItemOut = Tick>,
    L: 'static + LatticeRepr + Merge<Update>,
    Update: 'static + LatticeRepr,
    L::Repr: Default,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    type InputHandoffs = <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended;
    type Build = BatchPullBuild<PrevBuf::Build, PrevStream::Build, L, Update, Tick>;

    fn make_parts(self, ctx: &mut Context) -> (Self::InputHandoffs, Self::Build) {
        let (connect_a, build_a) = self.prev_a.make_parts(ctx);
        let (connect_b, build_b) = self.prev_b.make_parts(ctx);
        let connect = connect_a.extend(connect_b);
        let build = BatchPullBuild::new(build_a, build_b);
        (connect, build)
    }
}
