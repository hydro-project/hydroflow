use super::PullConnect;

use crate::scheduled::handoff::{HandoffList, HandoffListSplit};
use crate::scheduled::type_list::Extend;

pub struct BinaryPullConnect<PrevA, PrevB>
where
    PrevA: PullConnect,
    PrevB: PullConnect,
{
    prev_a: PrevA,
    prev_b: PrevB,
}

impl<PrevA, PrevB> BinaryPullConnect<PrevA, PrevB>
where
    PrevA: PullConnect,
    PrevB: PullConnect,
{
    pub fn new(prev_a: PrevA, prev_b: PrevB) -> Self {
        Self { prev_a, prev_b }
    }
}

impl<PrevA, PrevB> PullConnect for BinaryPullConnect<PrevA, PrevB>
where
    PrevA: PullConnect,
    PrevB: PullConnect,
    PrevA::InputHandoffs: Extend<PrevB::InputHandoffs>,
    <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended:
        HandoffList + HandoffListSplit<PrevA::InputHandoffs, Suffix = PrevB::InputHandoffs>,
{
    type InputHandoffs = <PrevA::InputHandoffs as Extend<PrevB::InputHandoffs>>::Extended;
    fn connect(self, ports: <Self::InputHandoffs as HandoffList>::InputPort) {
        let (ports_a, ports_b) =
            <Self::InputHandoffs as HandoffListSplit<_>>::split_input_port(ports);
        self.prev_a.connect(ports_a);
        self.prev_b.connect(ports_b);
    }
}
