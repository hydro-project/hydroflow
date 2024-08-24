use std::marker::PhantomData;

use crate::location::Location;
use crate::stream::CycleCollection;

/// Represents a fixpoint cycle in the graph that will be fulfilled
/// by a stream that is not yet known.
///
/// See [`FlowBuilder`] for an explainer on the type parameters.
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
