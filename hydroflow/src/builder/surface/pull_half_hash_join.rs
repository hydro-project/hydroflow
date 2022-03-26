use super::{AssembleFlowGraph, BaseSurface, PullSurface};

use std::hash::Hash;
use std::marker::PhantomData;

use crate::builder::build::pull_half_hash_join::HalfHashJoinPullBuild;
use crate::lang::lattice::{LatticeRepr, Merge};
use crate::scheduled::graph::NodeId;
use crate::scheduled::handoff::handoff_list::{PortList, PortListSplit};
use crate::scheduled::port::RECV;
use crate::scheduled::type_list::Extend;

pub struct HalfHashJoinPullSurface<PrevStream, PrevBuf, L, Update>
where
    PrevBuf: PullSurface,
    PrevStream: PullSurface,
    L: LatticeRepr,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    prev_stream: PrevStream,
    prev_buf: PrevBuf,
    _marker: PhantomData<(L, Update)>,
}
impl<PrevBuf, PrevStream, Key, L, Update, StreamVal>
    HalfHashJoinPullSurface<PrevStream, PrevBuf, L, Update>
where
    PrevBuf: PullSurface<ItemOut = (Key, Update::Repr)>,
    PrevStream: PullSurface<ItemOut = (Key, StreamVal)>,
    Key: 'static + Eq + Hash,
    L: 'static + LatticeRepr + Merge<Update>,
    Update: 'static + LatticeRepr,
    StreamVal: 'static,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    pub fn new(prev_stream: PrevStream, prev_buf: PrevBuf) -> Self {
        Self {
            prev_stream,
            prev_buf,
            _marker: PhantomData,
        }
    }
}
impl<PrevBuf, PrevStream, Key, L, Update, StreamVal> AssembleFlowGraph
    for HalfHashJoinPullSurface<PrevStream, PrevBuf, L, Update>
where
    PrevBuf: PullSurface<ItemOut = (Key, Update::Repr)> + AssembleFlowGraph,
    PrevStream: PullSurface<ItemOut = (Key, StreamVal)> + AssembleFlowGraph,
    Key: 'static + Eq + Hash,
    L: 'static + LatticeRepr + Merge<Update>,
    Update: 'static + LatticeRepr,
    StreamVal: 'static,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> NodeId {
        let my_id = e.add_node("HalfHashJoin");
        let prev_a_id = self.prev_buf.insert_dep(e);
        let prev_b_id = self.prev_stream.insert_dep(e);
        e.add_edge((prev_a_id, my_id));
        e.add_edge((prev_b_id, my_id));
        my_id
    }
}

impl<PrevBuf, PrevStream, Key, L, Update, StreamVal> BaseSurface
    for HalfHashJoinPullSurface<PrevStream, PrevBuf, L, Update>
where
    PrevBuf: PullSurface<ItemOut = (Key, Update::Repr)>,
    PrevStream: PullSurface<ItemOut = (Key, StreamVal)>,
    Key: 'static + Eq + Hash,
    L: 'static + LatticeRepr + Merge<Update>,
    Update: 'static + LatticeRepr,
    StreamVal: 'static,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    type ItemOut = (Key, StreamVal, L::Repr);
}

impl<PrevBuf, PrevStream, Key, L, Update, StreamVal> PullSurface
    for HalfHashJoinPullSurface<PrevStream, PrevBuf, L, Update>
where
    PrevBuf: PullSurface<ItemOut = (Key, Update::Repr)>,
    PrevStream: PullSurface<ItemOut = (Key, StreamVal)>,
    Key: 'static + Eq + Hash,
    L: 'static + LatticeRepr + Merge<Update>,
    L::Repr: Default,
    Update: 'static + LatticeRepr,
    StreamVal: 'static,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    type InputHandoffs = <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended;
    type Build =
        HalfHashJoinPullBuild<PrevBuf::Build, PrevStream::Build, Key, L, Update, StreamVal>;

    fn into_parts(self) -> (Self::InputHandoffs, Self::Build) {
        let (connect_a, build_a) = self.prev_buf.into_parts();
        let (connect_b, build_b) = self.prev_stream.into_parts();
        let connect = connect_a.extend(connect_b);
        let build = HalfHashJoinPullBuild::new(build_a, build_b);
        (connect, build)
    }
}
