use super::PushSurfaceReversed;

use crate::builder::build::push_tee::TeePushBuild;
use crate::scheduled::handoff::handoff_list::{BasePortListSplit, SendPortList};
use crate::scheduled::type_list::Extend;

pub struct TeePushSurfaceReversed<NextA, NextB>
where
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended: SendPortList
        + BasePortListSplit<NextA::OutputHandoffs, false, Suffix = NextB::OutputHandoffs>,
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
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended: SendPortList
        + BasePortListSplit<NextA::OutputHandoffs, false, Suffix = NextB::OutputHandoffs>,
{
    pub fn new(next_a: NextA, next_b: NextB) -> Self {
        Self { next_a, next_b }
    }
}

impl<NextA, NextB> PushSurfaceReversed for TeePushSurfaceReversed<NextA, NextB>
where
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended: SendPortList
        + BasePortListSplit<NextA::OutputHandoffs, false, Suffix = NextB::OutputHandoffs>,
{
    type OutputHandoffs = <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended;

    type ItemIn = NextA::ItemIn;

    type Build = TeePushBuild<NextA::Build, NextB::Build>;

    fn into_build(self) -> Self::Build {
        TeePushBuild::new(self.next_a.into_build(), self.next_b.into_build())
    }
}
