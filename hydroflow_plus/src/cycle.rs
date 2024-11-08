use std::marker::PhantomData;

use crate::location::Location;

pub enum ForwardRefMarker {}
pub enum TickCycleMarker {}

pub trait DeferTick {
    fn defer_tick(self) -> Self;
}

pub trait CycleComplete<'a, T> {
    fn complete(self, ident: syn::Ident);
}

pub trait CycleCollection<'a, T>: CycleComplete<'a, T> {
    type Location: Location<'a>;

    fn create_source(ident: syn::Ident, location: Self::Location) -> Self;
}

pub trait CycleCollectionWithInitial<'a, T>: CycleComplete<'a, T> {
    type Location: Location<'a>;

    fn create_source(ident: syn::Ident, initial: Self, location: Self::Location) -> Self;
}

/// Represents a forward reference in the graph that will be fulfilled
/// by a stream that is not yet known.
///
/// See [`crate::FlowBuilder`] for an explainer on the type parameters.
pub struct ForwardRef<'a, S: CycleComplete<'a, ForwardRefMarker>> {
    pub(crate) ident: syn::Ident,
    pub(crate) _phantom: PhantomData<(&'a mut &'a (), S)>,
}

impl<'a, S: CycleComplete<'a, ForwardRefMarker>> ForwardRef<'a, S> {
    pub fn complete(self, stream: S) {
        let ident = self.ident;
        S::complete(stream, ident)
    }
}

pub struct TickCycle<'a, S: CycleComplete<'a, TickCycleMarker> + DeferTick> {
    pub(crate) ident: syn::Ident,
    pub(crate) _phantom: PhantomData<(&'a mut &'a (), S)>,
}

impl<'a, S: CycleComplete<'a, TickCycleMarker> + DeferTick> TickCycle<'a, S> {
    pub fn complete_next_tick(self, stream: S) {
        let ident = self.ident;
        S::complete(stream.defer_tick(), ident)
    }
}
