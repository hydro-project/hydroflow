use super::context::Context;

/**
 * Represents a compiled subgraph. Used internally by [Dataflow] to erase the input/output [Handoff] types.
 */
pub(crate) trait Subgraph {
    // TODO: pass in some scheduling info?
    fn run(&mut self, context: &mut Context);
}
impl<F> Subgraph for F
where
    F: FnMut(&mut Context),
{
    fn run(&mut self, context: &mut Context) {
        (self)(context);
    }
}
