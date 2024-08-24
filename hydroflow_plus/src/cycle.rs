use std::marker::PhantomData;

use crate::builder::FlowLeaves;
use crate::location::{Location, LocationId};

pub trait CycleCollection<'a> {
    type Location: Location;

    fn create_source(ident: syn::Ident, ir_leaves: FlowLeaves<'a>, l: LocationId) -> Self;

    fn complete(self, ident: syn::Ident);
}

/// Represents a fixpoint cycle in the graph that will be fulfilled
/// by a stream that is not yet known.
///
/// See [`crate::FlowBuilder`] for an explainer on the type parameters.
pub struct HfCycle<'a, S: CycleCollection<'a>> {
    pub(crate) ident: syn::Ident,
    pub(crate) _phantom: PhantomData<(&'a mut &'a (), S)>,
}

impl<'a, S: CycleCollection<'a>> HfCycle<'a, S> {
    pub fn complete(self, stream: S) {
        let ident = self.ident;
        S::complete(stream, ident)
    }
}
