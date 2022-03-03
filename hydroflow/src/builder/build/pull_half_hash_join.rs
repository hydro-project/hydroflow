use super::{PullBuild, PullBuildBase};

use std::hash::Hash;
use std::marker::PhantomData;

use crate::compiled::pull::{HalfHashJoin, HalfHashJoinState};
use crate::lang::lattice::{LatticeRepr, Merge};
use crate::scheduled::context::Context;
use crate::scheduled::handoff::handoff_list::{PortList, PortListSplit};
use crate::scheduled::port::RECV;
use crate::scheduled::type_list::Extend;

pub struct HalfHashJoinPullBuild<PrevBuf, PrevStream, Key, L, Update, StreamVal>
where
    PrevBuf: PullBuild<ItemOut = (Key, Update::Repr)>,
    PrevStream: PullBuild<ItemOut = (Key, StreamVal)>,
    Key: 'static + Eq,
    L: 'static + LatticeRepr + Merge<Update>,
    Update: 'static + LatticeRepr,
    StreamVal: 'static,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    prev_a: PrevBuf,
    prev_b: PrevStream,
    state: HalfHashJoinState<Key, L>,
    _marker: PhantomData<Update>,
}
impl<PrevBuf, PrevStream, Key, L, Update, StreamVal>
    HalfHashJoinPullBuild<PrevBuf, PrevStream, Key, L, Update, StreamVal>
where
    PrevBuf: PullBuild<ItemOut = (Key, Update::Repr)>,
    PrevStream: PullBuild<ItemOut = (Key, StreamVal)>,
    Key: 'static + Eq + Hash,
    L: 'static + LatticeRepr + Merge<Update>,
    Update: 'static + LatticeRepr,
    StreamVal: 'static,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    pub fn new(prev_a: PrevBuf, prev_b: PrevStream) -> Self {
        Self {
            prev_a,
            prev_b,
            state: Default::default(),
            _marker: PhantomData,
        }
    }
}

impl<PrevBuf, PrevStream, Key, L, Update, StreamVal> PullBuildBase
    for HalfHashJoinPullBuild<PrevBuf, PrevStream, Key, L, Update, StreamVal>
where
    PrevBuf: PullBuild<ItemOut = (Key, Update::Repr)>,
    PrevStream: PullBuild<ItemOut = (Key, StreamVal)>,
    Key: 'static + Eq + Hash,
    L: 'static + LatticeRepr + Merge<Update>,
    L::Repr: Default,
    Update: 'static + LatticeRepr,
    StreamVal: 'static,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    type ItemOut = (Key, StreamVal, L::Repr);
    type Build<'slf, 'hof> = HalfHashJoin<
        'slf,
        Key,
        PrevBuf::Build<'slf, 'hof>,
        L,
        Update,
        PrevStream::Build<'slf, 'hof>,
        StreamVal,
    >;
}

impl<PrevBuf, PrevStream, Key, L, Update, StreamVal> PullBuild
    for HalfHashJoinPullBuild<PrevBuf, PrevStream, Key, L, Update, StreamVal>
where
    PrevBuf: PullBuild<ItemOut = (Key, Update::Repr)>,
    PrevStream: PullBuild<ItemOut = (Key, StreamVal)>,
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

    fn build<'slf, 'hof>(
        &'slf mut self,
        context: &Context<'_>,
        input: <Self::InputHandoffs as PortList<RECV>>::Ctx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        let (input_a, input_b) = <Self::InputHandoffs as PortListSplit<_, _>>::split_ctx(input);
        let iter_a = self.prev_a.build(context, input_a);
        let iter_b = self.prev_b.build(context, input_b);
        HalfHashJoin::new(iter_a, iter_b, &mut self.state)
    }
}
