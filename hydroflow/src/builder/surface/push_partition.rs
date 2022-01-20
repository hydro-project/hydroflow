use super::PushSurfaceReversed;

use crate::builder::build::push_partition::PartitionPushBuild;
use crate::builder::connect::BinaryPushConnect;
use crate::scheduled::handoff::{HandoffList, HandoffListSplit};
use crate::scheduled::type_list::Extend;

pub struct PartitionPushSurfaceReversed<NextA, NextB, Func>
where
    Func: Fn(&NextA::ItemIn) -> bool,
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,
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
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        HandoffList + HandoffListSplit<NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    type OutputHandoffs = <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended;

    type ItemIn = NextA::ItemIn;

    type Connect = BinaryPushConnect<NextA::Connect, NextB::Connect>;
    type Build = PartitionPushBuild<NextA::Build, NextB::Build, Func>;

    fn into_parts(self) -> (Self::Connect, Self::Build) {
        let (connect_a, build_a) = self.next_a.into_parts();
        let (connect_b, build_b) = self.next_b.into_parts();
        let connect = BinaryPushConnect::new(connect_a, connect_b);
        let build = PartitionPushBuild::new(self.func, build_a, build_b);
        (connect, build)
    }
}
