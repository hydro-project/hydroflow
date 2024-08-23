use std::marker::PhantomData;

use crate::builder::FlowLeaves;
use crate::ir::{HfPlusLeaf, HfPlusNode};
use crate::location::{Location, LocationId};
use crate::stream::{NoTick, Tick};
use crate::Stream;

/// Represents a fixpoint cycle in the graph that will be fulfilled
/// by a stream that is not yet known.
///
/// See [`Stream`] for an explainer on the type parameters.
pub struct HfCycle<'a, T, W, C, N: Location> {
    pub(crate) ident: syn::Ident,
    pub(crate) location_kind: LocationId,
    pub(crate) ir_leaves: FlowLeaves<'a>,
    pub(crate) _phantom: PhantomData<(N, &'a mut &'a (), T, W, C)>,
}

impl<'a, T, W, N: Location> HfCycle<'a, T, W, Tick, N> {
    pub fn complete(self, stream: Stream<'a, T, W, Tick, N>) {
        let ident = self.ident;

        self.ir_leaves.borrow_mut().as_mut().expect("Attempted to add a cycle to a flow that has already been finalized. No cycles can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind,
            input: Box::new(stream.ir_node.into_inner()),
        });
    }
}

impl<'a, T, W, N: Location> HfCycle<'a, T, W, NoTick, N> {
    pub fn complete(self, stream: Stream<'a, T, W, NoTick, N>) {
        let ident = self.ident;

        self.ir_leaves.borrow_mut().as_mut().expect("Attempted to add a cycle to a flow that has already been finalized. No cycles can be added after the flow has been compiled.").push(HfPlusLeaf::CycleSink {
            ident,
            location_kind: self.location_kind,
            input: Box::new(HfPlusNode::Delta(
                Box::new(stream.ir_node.into_inner())
            )),
        });
    }
}
