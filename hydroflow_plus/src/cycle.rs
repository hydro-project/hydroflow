use std::cell::RefCell;
use std::marker::PhantomData;

use crate::ir::HfPlusNode;
use crate::location::Location;
use crate::Stream;

/// Represents a fixpoint cycle in the graph that will be fulfilled
/// by a stream that is not yet known.
///
/// See [`Stream`] for an explainer on the type parameters.
pub struct HfCycle<'a, T, W, N: Location<'a>> {
    pub(crate) ident: syn::Ident,
    pub(crate) node: N,
    pub(crate) ir_leaves: &'a RefCell<Vec<HfPlusNode>>,
    pub(crate) _phantom: PhantomData<(T, W)>,
}

impl<'a, T, W, N: Location<'a>> HfCycle<'a, T, W, N> {
    pub fn complete(self, stream: Stream<'a, T, W, N>) {
        let ident = self.ident;

        self.ir_leaves.borrow_mut().push(HfPlusNode::CycleSink {
            ident,
            location_id: self.node.id(),
            input: Box::new(stream.ir_node.into_inner()),
        });
    }
}
