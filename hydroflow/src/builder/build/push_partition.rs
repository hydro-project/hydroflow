use super::{PushBuild, PushBuildBase};

use crate::compiled::partition::Partition;
use crate::scheduled::handoff::handoff_list::{PortList, PortListSplit};
use crate::scheduled::port::SEND;
use crate::scheduled::type_list::Extend;

pub struct PartitionPushBuild<NextA, NextB, Func>
where
    Func: Fn(&NextA::ItemIn) -> bool,
    NextA: PushBuild,
    NextB: PushBuild<ItemIn = NextA::ItemIn>,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    func: Func,
    next_a: NextA,
    next_b: NextB,
}
impl<Func, NextA, NextB> PartitionPushBuild<NextA, NextB, Func>
where
    Func: Fn(&NextA::ItemIn) -> bool,
    NextA: PushBuild,
    NextB: PushBuild<ItemIn = NextA::ItemIn>,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    pub fn new(func: Func, next_a: NextA, next_b: NextB) -> Self {
        Self {
            func,
            next_a,
            next_b,
        }
    }
}

#[allow(type_alias_bounds)]
type PushBuildImpl<'slf, 'hof, NextA, NextB, Func>
where
    NextA: PushBuild,
    NextB: PushBuild<ItemIn = NextA::ItemIn>,
= Partition<
    NextA::ItemIn,
    impl Fn(&NextA::ItemIn) -> bool,
    NextA::Build<'slf, 'hof>,
    NextB::Build<'slf, 'hof>,
>;

impl<NextA, NextB, Func> PushBuildBase for PartitionPushBuild<NextA, NextB, Func>
where
    Func: Fn(&NextA::ItemIn) -> bool,
    NextA: PushBuild,
    NextB: PushBuild<ItemIn = NextA::ItemIn>,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    type ItemIn = NextA::ItemIn;
    type Build<'slf, 'hof> = PushBuildImpl<'slf, 'hof, NextA, NextB, Func>;
}

impl<NextA, NextB, Func> PushBuild for PartitionPushBuild<NextA, NextB, Func>
where
    Func: Fn(&NextA::ItemIn) -> bool,
    NextA: PushBuild,
    NextB: PushBuild<ItemIn = NextA::ItemIn>,

    NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
    <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
        PortList<SEND> + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
{
    type OutputHandoffs = <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended;

    fn build<'slf, 'hof>(
        &'slf mut self,
        input: <Self::OutputHandoffs as PortList<SEND>>::Ctx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        let (input_a, input_b) = <Self::OutputHandoffs as PortListSplit<_, _>>::split_ctx(input);
        let build_a = self.next_a.build(input_a);
        let build_b = self.next_b.build(input_b);
        Partition::new(|x| (self.func)(x), build_a, build_b)
    }
}
