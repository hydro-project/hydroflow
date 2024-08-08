use std::marker::PhantomData;

use crate::builder::FlowLeaves;
use crate::ir::HfPlusLeaf;
use crate::location::Location;
use crate::Stream;

/// Represents a fixpoint cycle in the graph that will be fulfilled
/// by a stream that is not yet known.
///
/// See [`Stream`] for an explainer on the type parameters.
pub struct HfCycle<'a, T, W, N: Location + Clone> {
    pub(crate) ident: syn::Ident,
    pub(crate) node: N,
    pub(crate) ir_leaves: FlowLeaves<'a>,
    pub(crate) _phantom: PhantomData<(&'a mut &'a (), T, W)>,
}

impl<'a, T, W, N: Location + Clone> HfCycle<'a, T, W, N> {
    pub fn complete(self, stream: Stream<'a, T, W, N>) {
        let ident = self.ident;

        self.ir_leaves.borrow_mut().as_mut().expect("Attempted to add a cycle to a flow that has already been finalized. No cycles can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_id: self.node.id(),
            input: Box::new(stream.ir_node.into_inner()),
        });
    }
}
