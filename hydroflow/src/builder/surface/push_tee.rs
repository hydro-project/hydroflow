use super::PushSurfaceReversed;

use crate::builder::build::push_tee::TeePushBuild;
use crate::builder::connect::BinaryPushConnect;
use crate::scheduled::handoff::{HandoffList, HandoffListSplit};
use crate::scheduled::type_list::Extend;

pub struct TeePushSurfaceReversed<NextA, NextB>
where
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,
{
    next_a: NextA,
    next_b: NextB,
}
impl<NextA, NextB> TeePushSurfaceReversed<NextA, NextB>
where
    NextA: PushSurfaceReversed,
    NextB: PushSurfaceReversed<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,
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
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        HandoffList + HandoffListSplit<NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    type OutputHandoffs = <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended;

    type ItemIn = NextA::ItemIn;

    type Connect = BinaryPushConnect<NextA::Connect, NextB::Connect>;
    type Build = TeePushBuild<NextA::Build, NextB::Build>;

    fn into_parts(self) -> (Self::Connect, Self::Build) {
        let (connect_a, build_a) = self.next_a.into_parts();
        let (connect_b, build_b) = self.next_b.into_parts();
        let connect = BinaryPushConnect::new(connect_a, connect_b);
        let build = TeePushBuild::new(build_a, build_b);
        (connect, build)
    }
}
