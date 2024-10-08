use std::marker::PhantomData;

use crate::builder::FlowState;
use crate::location::{Location, LocationId};
use crate::Tick;

pub trait DeferTick {
    fn defer_tick(self) -> Self;
}

pub trait CycleComplete<'a, T> {
    fn complete(self, ident: syn::Ident);
}

pub trait CycleCollection<'a, T>: CycleComplete<'a, T> {
    type Location: Location<'a>;

    fn create_source(ident: syn::Ident, flow_state: FlowState, l: LocationId) -> Self;
}

pub trait CycleCollectionWithInitial<'a, T>: CycleComplete<'a, T> {
    type Location: Location<'a>;

    fn create_source(
        ident: syn::Ident,
        flow_state: FlowState,
        initial: Self,
        l: LocationId,
    ) -> Self;
}

/// Represents a forward reference in the graph that will be fulfilled
/// by a stream that is not yet known.
///
/// See [`crate::FlowBuilder`] for an explainer on the type parameters.
pub struct HfForwardRef<'a, T, S: CycleComplete<'a, T>> {
    pub(crate) ident: syn::Ident,
    pub(crate) _phantom: PhantomData<(&'a mut &'a (), T, S)>,
}

impl<'a, T, S: CycleComplete<'a, T>> HfForwardRef<'a, T, S> {
    pub fn complete(self, stream: S) {
        let ident = self.ident;
        S::complete(stream, ident)
    }
}

pub struct HfCycle<'a, S: CycleComplete<'a, Tick> + DeferTick> {
    pub(crate) ident: syn::Ident,
    pub(crate) _phantom: PhantomData<(&'a mut &'a (), S)>,
}

impl<'a, S: CycleComplete<'a, Tick> + DeferTick> HfCycle<'a, S> {
    pub fn complete_next_tick(self, stream: S) {
        let ident = self.ident;
        S::complete(stream.defer_tick(), ident)
    }
}
