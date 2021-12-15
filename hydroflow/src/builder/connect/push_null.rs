use super::PushConnect;

use crate::scheduled::handoff::HandoffList;
use crate::tt;

#[derive(Default)]
pub struct NullPushConnect;

impl NullPushConnect {
    pub fn new() -> Self {
        Default::default()
    }
}

impl PushConnect for NullPushConnect {
    type OutputHandoffs = tt!();
    fn connect(self, (): <Self::OutputHandoffs as HandoffList>::OutputPort) {}
}
