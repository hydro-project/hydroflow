use super::PushSurfaceReversed;

use crate::builder::build::push_partition::PartitionPushBuild;
use crate::scheduled::handoff::handoff_list::{BasePortListSplit, SendPortList};
use crate::scheduled::type_list::Extend;

pub struct PartitionPushSurfaceReversed<NextA, NextB, Func>
where
    Func: Fn(&NextA::ItemIn) -> bool,
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended: SendPortList
        + BasePortListSplit<NextA::OutputHandoffs, false, Suffix = NextB::OutputHandoffs>,
{
    func: Func,
    next_a: NextA,
    next_b: NextB,
}
impl<NextA, NextB, Func> PartitionPushSurfaceReversed<NextA, NextB, Func>
where
    Func: Fn(&NextA::ItemIn) -> bool,
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended: SendPortList
        + BasePortListSplit<NextA::OutputHandoffs, false, Suffix = NextB::OutputHandoffs>,
{
    pub fn new(func: Func, next_a: NextA, next_b: NextB) -> Self {
        Self {
            func,
            next_a,
            next_b,
        }
    }
}

impl<NextA, NextB, Func> PushSurfaceReversed for PartitionPushSurfaceReversed<NextA, NextB, Func>
where
    Func: Fn(&NextA::ItemIn) -> bool,
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended: SendPortList
        + BasePortListSplit<NextA::OutputHandoffs, false, Suffix = NextB::OutputHandoffs>,
{
    type OutputHandoffs = <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended;

    type ItemIn = NextA::ItemIn;

    type Build = PartitionPushBuild<NextA::Build, NextB::Build, Func>;

    fn into_build(self) -> Self::Build {
        PartitionPushBuild::new(self.next_a.into_build(), self.next_b.into_build())
    }
}
