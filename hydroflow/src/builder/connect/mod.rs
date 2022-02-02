//! Internal "connects" which connect input/output ports to implement the Surface API. For more info see [super].

mod pull_binary;
mod pull_handoff;
mod pull_null;
mod push_binary;
mod push_handoff;
mod push_null;

pub use pull_binary::BinaryPullConnect;
pub use pull_handoff::HandoffPullConnect;
pub use pull_null::NullPullConnect;
pub use push_binary::BinaryPushConnect;
pub use push_handoff::HandoffPushConnect;
pub use push_null::NullPushConnect;

use crate::scheduled::handoff::HandoffList;

pub trait PullConnect {
    type InputHandoffs: HandoffList;
    fn connect(self, ports: <Self::InputHandoffs as HandoffList>::InputPort);
}

pub trait PushConnect {
    type OutputHandoffs: HandoffList;
    fn connect(self, ports: <Self::OutputHandoffs as HandoffList>::OutputPort);
}
