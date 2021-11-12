use crate::scheduled::{HandoffData, StateData};

/**
 * Represents a compiled subgraph. Used internally by [Dataflow] to erase the input/output [Handoff] types.
 */
pub(crate) trait Subgraph {
    // TODO: pass in some scheduling info?
    fn run(&mut self, handoffs: &[HandoffData], states: &[StateData]);
}
impl<F> Subgraph for F
where
    F: FnMut(&[HandoffData], &[StateData]),
{
    fn run(&mut self, handoffs: &[HandoffData], states: &[StateData]) {
        (self)(handoffs, states);
    }
}
