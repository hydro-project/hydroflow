use crate::scheduled::HandoffData;

/**
 * Represents a compiled subgraph. Used internally by [Dataflow] to erase the input/output [Handoff] types.
 */
pub(crate) trait Subgraph {
    // TODO: pass in some scheduling info?
    fn run(&mut self, handoffs: &[HandoffData]);
}
impl<F> Subgraph for F
where
    F: FnMut(&[HandoffData]),
{
    fn run(&mut self, handoffs: &[HandoffData]) {
        (self)(handoffs);
    }
}
