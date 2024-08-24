use std::marker::PhantomData;

use crate::builder::FlowLeaves;
use crate::location::Location;

pub trait CycleCollection<'a, L, C> {
    fn create_source(ident: syn::Ident, ir_leaves: FlowLeaves<'a>, l: &L) -> Self;

    fn complete(self, ident: syn::Ident);
}

/// Represents a fixpoint cycle in the graph that will be fulfilled
/// by a stream that is not yet known.
///
/// See [`crate::FlowBuilder`] for an explainer on the type parameters.
pub struct HfCycle<'a, N: Location, C, S: CycleCollection<'a, N, C>> {
    pub(crate) ident: syn::Ident,
    pub(crate) _phantom: PhantomData<(&'a mut &'a (), N, C, S)>,
}

impl<'a, N: Location, C, S: CycleCollection<'a, N, C>> HfCycle<'a, N, C, S> {
    pub fn complete(self, stream: S) {
        let ident = self.ident;
        S::complete(stream, ident)
    }
}
