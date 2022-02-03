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
        + BasePortListSplit<NextA::OutputHandoffs, true, Suffix = NextB::OutputHandoffs>,
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
        + BasePortListSplit<NextA::OutputHandoffs, true, Suffix = NextB::OutputHandoffs>,
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
        + BasePortListSplit<NextA::OutputHandoffs, true, Suffix = NextB::OutputHandoffs>,
{
    type ItemIn = NextA::ItemIn;

    type OutputHandoffs = <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended;
    type Build = PartitionPushBuild<NextA::Build, NextB::Build, Func>;

    fn into_parts(self) -> (Self::OutputHandoffs, Self::Build) {
        let (connect_a, build_a) = self.next_a.into_parts();
        let (connect_b, build_b) = self.next_b.into_parts();
        let connect = connect_a.extend(connect_b);
        let build = PartitionPushBuild::new(self.func, build_a, build_b);
        (connect, build)
    }
}
