use super::{PushBuild, PushBuildBase};

use crate::compiled::tee::Tee;
use crate::scheduled::handoff::{HandoffList, HandoffListSplit};
use crate::scheduled::type_list::Extend;

pub struct TeePushBuild<NextA, NextB>
where
    NextA: PushBuild,
    NextB: PushBuild<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,
{
    next_a: NextA,
    next_b: NextB,
}
impl<NextA, NextB> TeePushBuild<NextA, NextB>
where
    NextA: PushBuild,
    NextB: PushBuild<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,
{
    pub fn new(next_a: NextA, next_b: NextB) -> Self {
        Self { next_a, next_b }
    }
}

impl<NextA, NextB> PushBuildBase for TeePushBuild<NextA, NextB>
where
    NextA: PushBuild,
    NextB: PushBuild<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,
{
    type ItemIn = NextA::ItemIn;
    type Build<'slf, 'hof> = Tee<Self::ItemIn, NextA::Build<'slf, 'hof>, NextB::Build<'slf, 'hof>>;
}

impl<NextA, NextB> PushBuild for TeePushBuild<NextA, NextB>
where
    NextA: PushBuild,
    NextB: PushBuild<ItemIn = NextA::ItemIn>,
    NextA::ItemIn: Clone,
    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        HandoffList + HandoffListSplit<NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    type OutputHandoffs = <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended;

    fn build<'slf, 'hof>(
        &'slf mut self,
        input: <Self::OutputHandoffs as HandoffList>::SendCtx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        let (input_a, input_b) =
            <Self::OutputHandoffs as HandoffListSplit<_>>::split_send_ctx(input);
        let build_a = self.next_a.build(input_a);
        let build_b = self.next_b.build(input_b);
        Tee::new(build_a, build_b)
    }
}
