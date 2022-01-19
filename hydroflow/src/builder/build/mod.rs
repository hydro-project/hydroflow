//! Internal "subgraph builders" to implement the Surface API. For more info see [super].

pub mod pull_chain;
pub mod pull_cross_join;
pub mod pull_filter;
pub mod pull_flatten;
pub mod pull_handoff;
pub mod pull_join;
pub mod pull_map;

pub mod push_filter;
pub mod push_flatten;
pub mod push_for_each;
pub mod push_handoff;
pub mod push_map;
pub mod push_partition;
pub mod push_tee;

use crate::compiled::Pusherator;
use crate::scheduled::handoff::HandoffList;

pub trait PullBuildBase {
    type ItemOut;
    type Build<'slf, 'inp>: Iterator<Item = Self::ItemOut>;
}
pub trait PullBuild: PullBuildBase {
    type InputHandoffs: HandoffList;

    /// Builds the iterator for a single run of the subgraph.
    fn build<'slf, 'hof>(
        &'slf mut self,
        handoffs: <Self::InputHandoffs as HandoffList>::RecvCtx<'hof>,
    ) -> Self::Build<'slf, 'hof>;
}

pub trait PushBuildBase {
    type ItemIn;
    type Build<'slf, 'hof>: Pusherator<Item = Self::ItemIn>;
}
pub trait PushBuild: PushBuildBase {
    type OutputHandoffs: HandoffList;

    /// Builds the pusherator for a single run of the subgraph.
    fn build<'slf, 'hof>(
        &'slf mut self,
        handoffs: <Self::OutputHandoffs as HandoffList>::SendCtx<'hof>,
    ) -> Self::Build<'slf, 'hof>;
}
