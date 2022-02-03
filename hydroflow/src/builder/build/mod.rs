//! Internal "subgraph builders" to implement the Surface API. For more info see [super].

pub mod pull_chain;
pub mod pull_cross_join;
pub mod pull_filter;
pub mod pull_filter_map;
pub mod pull_flatten;
pub mod pull_handoff;
pub mod pull_iter;
pub mod pull_join;
pub mod pull_map;

pub mod push_filter;
pub mod push_filter_map;
pub mod push_flatten;
pub mod push_for_each;
pub mod push_handoff;
pub mod push_map;
pub mod push_partition;
pub mod push_tee;

use crate::compiled::Pusherator;
use crate::scheduled::handoff::handoff_list::{BasePortList, RecvPortList, SendPortList};

pub trait PullBuildBase {
    type ItemOut;
    type Build<'slf, 'inp>: Iterator<Item = Self::ItemOut>;
}
pub trait PullBuild: PullBuildBase {
    type InputHandoffs: RecvPortList;

    /// Builds the iterator for a single run of the subgraph.
    fn build<'slf, 'hof>(
        &'slf mut self,
        handoffs: <Self::InputHandoffs as BasePortList<false>>::Ctx<'hof>,
    ) -> Self::Build<'slf, 'hof>;
}

pub trait PushBuildBase {
    type ItemIn;
    type Build<'slf, 'hof>: Pusherator<Item = Self::ItemIn>;
}
pub trait PushBuild: PushBuildBase {
    type OutputHandoffs: SendPortList;

    /// Builds the pusherator for a single run of the subgraph.
    fn build<'slf, 'hof>(
        &'slf mut self,
        handoffs: <Self::OutputHandoffs as BasePortList<true>>::Ctx<'hof>,
    ) -> Self::Build<'slf, 'hof>;
}
