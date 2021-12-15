use super::PushConnect;

use crate::scheduled::handoff::{HandoffList, HandoffListSplit};
use crate::scheduled::type_list::Extend;

pub struct BinaryPushConnect<PrevA, PrevB>
where
    PrevA: PushConnect,
    PrevB: PushConnect,
{
    prev_a: PrevA,
    prev_b: PrevB,
}

impl<PrevA, PrevB> BinaryPushConnect<PrevA, PrevB>
where
    PrevA: PushConnect,
    PrevB: PushConnect,
{
    pub fn new(prev_a: PrevA, prev_b: PrevB) -> Self {
        Self { prev_a, prev_b }
    }
}

impl<PrevA, PrevB> PushConnect for BinaryPushConnect<PrevA, PrevB>
where
    PrevA: PushConnect,
    PrevB: PushConnect,
    PrevA::OutputHandoffs: Extend<PrevB::OutputHandoffs>,
    <PrevA::OutputHandoffs as Extend<PrevB::OutputHandoffs>>::Extended:
        HandoffList + HandoffListSplit<PrevA::OutputHandoffs, Suffix = PrevB::OutputHandoffs>,
{
    type OutputHandoffs = <PrevA::OutputHandoffs as Extend<PrevB::OutputHandoffs>>::Extended;
    fn connect(self, ports: <Self::OutputHandoffs as HandoffList>::OutputPort) {
        let (ports_a, ports_b) =
            <Self::OutputHandoffs as HandoffListSplit<_>>::split_output_port(ports);
        self.prev_a.connect(ports_a);
        self.prev_b.connect(ports_b);
    }
}
