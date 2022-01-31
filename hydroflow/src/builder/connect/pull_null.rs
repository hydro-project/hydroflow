use super::PullConnect;

use crate::scheduled::handoff::HandoffList;

pub struct NullPullConnect;

impl PullConnect for NullPullConnect {
    type InputHandoffs = ();
    fn connect(self, _ports: <Self::InputHandoffs as HandoffList>::InputPort) {}
}
