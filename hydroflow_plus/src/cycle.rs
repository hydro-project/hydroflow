use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

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
    pub(crate) ir_leaves: Rc<RefCell<Vec<HfPlusLeaf>>>,
    pub(crate) _phantom: PhantomData<(&'a mut &'a (), T, W)>,
}

impl<'a, T, W, N: Location + Clone> HfCycle<'a, T, W, N> {
    pub fn complete(self, stream: Stream<'a, T, W, N>) {
        let ident = self.ident;

        self.ir_leaves.borrow_mut().push(HfPlusLeaf::CycleSink {
            ident,
            location_id: self.node.id(),
            input: Box::new(stream.ir_node.into_inner()),
        });
    }
}
