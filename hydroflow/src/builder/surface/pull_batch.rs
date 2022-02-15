use super::{BaseSurface, PullSurface};

use std::hash::Hash;

use crate::builder::build::pull_batch::BatchPullBuild;
use crate::scheduled::handoff::handoff_list::{PortList, PortListSplit};
use crate::scheduled::port::RECV;
use crate::scheduled::type_list::Extend;

pub struct BatchPullSurface<PrevBuf, PrevStream>
where
    PrevBuf: PullSurface,
    PrevStream: PullSurface,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    prev_a: PrevBuf,
    prev_b: PrevStream,
}
impl<PrevBuf, PrevStream, Key, BufVal, StreamVal> BatchPullSurface<PrevBuf, PrevStream>
where
    PrevBuf: PullSurface<ItemOut = (Key, BufVal)>,
    PrevStream: PullSurface<ItemOut = (Key, StreamVal)>,
    Key: 'static + Eq + Hash,
    BufVal: 'static,
    StreamVal: 'static,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    pub fn new(prev_a: PrevBuf, prev_b: PrevStream) -> Self {
        Self { prev_a, prev_b }
    }
}

impl<PrevBuf, PrevStream, Key, BufVal, StreamVal> BaseSurface
    for BatchPullSurface<PrevBuf, PrevStream>
where
    PrevBuf: PullSurface<ItemOut = (Key, BufVal)>,
    PrevStream: PullSurface<ItemOut = (Key, StreamVal)>,
    Key: 'static + Eq + Hash,
    BufVal: 'static,
    StreamVal: 'static,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    type ItemOut = (Key, StreamVal, Vec<BufVal>);
}

impl<PrevBuf, PrevStream, Key, BufVal, StreamVal> PullSurface
    for BatchPullSurface<PrevBuf, PrevStream>
where
    PrevBuf: PullSurface<ItemOut = (Key, BufVal)>,
    PrevStream: PullSurface<ItemOut = (Key, StreamVal)>,
    Key: 'static + Eq + Hash,
    BufVal: 'static,
    StreamVal: 'static,

    PrevBuf::InputHandoffs: Extend<PrevStream::InputHandoffs>,
    <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended: PortList<RECV>
        + PortListSplit<RECV, PrevBuf::InputHandoffs, Suffix = PrevStream::InputHandoffs>,
{
    type InputHandoffs = <PrevBuf::InputHandoffs as Extend<PrevStream::InputHandoffs>>::Extended;
    type Build = BatchPullBuild<PrevBuf::Build, PrevStream::Build, Key, BufVal, StreamVal>;

    fn into_parts(self) -> (Self::InputHandoffs, Self::Build) {
        let (connect_a, build_a) = self.prev_a.into_parts();
        let (connect_b, build_b) = self.prev_b.into_parts();
        let connect = connect_a.extend(connect_b);
        let build = BatchPullBuild::new(build_a, build_b);
        (connect, build)
    }
}
