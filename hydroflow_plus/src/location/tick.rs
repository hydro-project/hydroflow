use std::marker::PhantomData;

use proc_macro2::Span;
use stageleft::{q, QuotedWithContext};

use super::{Cluster, Location, LocationId, Process};
use crate::builder::FlowState;
use crate::cycle::{
    CycleCollection, CycleCollectionWithInitial, DeferTick, ForwardRef, ForwardRefMarker,
    TickCycle, TickCycleMarker,
};
use crate::ir::{HfPlusNode, HfPlusSource};
use crate::{Bounded, Optional, Singleton, Stream};

pub trait NoTick {}
impl<T> NoTick for Process<'_, T> {}
impl<T> NoTick for Cluster<'_, T> {}

pub trait NoTimestamp {}
impl<T> NoTimestamp for Process<'_, T> {}
impl<T> NoTimestamp for Cluster<'_, T> {}
impl<'a, L: Location<'a>> NoTimestamp for Tick<L> {}

#[derive(Clone)]
pub struct Timestamped<L> {
    pub(crate) tick: Tick<L>,
}

impl<'a, L: Location<'a>> Location<'a> for Timestamped<L> {
    type Root = L::Root;

    fn root(&self) -> Self::Root {
        self.tick.root()
    }

    fn id(&self) -> LocationId {
        self.tick.id()
    }

    fn flow_state(&self) -> &FlowState {
        self.tick.flow_state()
    }

    fn is_top_level() -> bool {
        L::is_top_level()
    }
}

impl<L> NoTick for Timestamped<L> {}

/// Marks the stream as being inside the single global clock domain.
#[derive(Clone)]
pub struct Tick<L> {
    pub(crate) id: usize,
    pub(crate) l: L,
}

impl<'a, L: Location<'a>> Location<'a> for Tick<L> {
    type Root = L::Root;

    fn root(&self) -> Self::Root {
        self.l.root()
    }

    fn id(&self) -> LocationId {
        LocationId::Tick(self.id, Box::new(self.l.id()))
    }

    fn flow_state(&self) -> &FlowState {
        self.l.flow_state()
    }

    fn is_top_level() -> bool {
        false
    }
}

impl<'a, L: Location<'a>> Tick<L> {
    pub fn outer(&self) -> &L {
        &self.l
    }

    pub fn spin_batch(
        &self,
        batch_size: impl QuotedWithContext<'a, usize, L> + Copy + 'a,
    ) -> Stream<(), Self, Bounded>
    where
        L: NoTick + NoTimestamp,
    {
        unsafe {
            // SAFETY: at runtime, `spin` produces a single value per tick,
            // so each batch is guaranteed to be the same size.
            self.l
                .spin()
                .flat_map_ordered(q!(move |_| 0..batch_size))
                .map(q!(|_| ()))
                .timestamped(self)
                .tick_batch()
        }
    }

    pub fn singleton<T: Clone>(
        &self,
        e: impl QuotedWithContext<'a, T, L>,
    ) -> Singleton<T, Self, Bounded>
    where
        L: NoTick,
    {
        unsafe {
            // SAFETY: a top-level singleton produces the same value each tick
            self.outer().singleton(e).timestamped(self).latest_tick()
        }
    }

    pub fn singleton_first_tick<T: Clone>(
        &self,
        e: impl QuotedWithContext<'a, T, Tick<L>>,
    ) -> Optional<T, Self, Bounded>
    where
        L: NoTick,
    {
        let e_arr = q!([e]);
        let e = e_arr.splice_untyped_ctx(self);

        Optional::new(
            self.clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Iter(e.into()),
                location_kind: self.l.id(),
            },
        )
    }

    pub fn forward_ref<S: CycleCollection<'a, ForwardRefMarker, Location = Self>>(
        &self,
    ) -> (ForwardRef<'a, S>, S)
    where
        L: NoTick,
    {
        let next_id = {
            let on_id = match self.l.id() {
                LocationId::Process(id) => id,
                LocationId::Cluster(id) => id,
                LocationId::Tick(_, _) => panic!(),
                LocationId::ExternalProcess(_) => panic!(),
            };

            let mut flow_state = self.flow_state().borrow_mut();
            let next_id_entry = flow_state.cycle_counts.entry(on_id).or_default();

            let id = *next_id_entry;
            *next_id_entry += 1;
            id
        };

        let ident = syn::Ident::new(&format!("cycle_{}", next_id), Span::call_site());

        (
            ForwardRef {
                ident: ident.clone(),
                expected_location: self.id(),
                _phantom: PhantomData,
            },
            S::create_source(ident, self.clone()),
        )
    }

    pub fn forward_ref_timestamped<
        S: CycleCollection<'a, ForwardRefMarker, Location = Timestamped<L>>,
    >(
        &self,
    ) -> (ForwardRef<'a, S>, S) {
        let next_id = {
            let on_id = match self.l.id() {
                LocationId::Process(id) => id,
                LocationId::Cluster(id) => id,
                LocationId::Tick(_, _) => panic!(),
                LocationId::ExternalProcess(_) => panic!(),
            };

            let mut flow_state = self.flow_state().borrow_mut();
            let next_id_entry = flow_state.cycle_counts.entry(on_id).or_default();

            let id = *next_id_entry;
            *next_id_entry += 1;
            id
        };

        let ident = syn::Ident::new(&format!("cycle_{}", next_id), Span::call_site());

        (
            ForwardRef {
                ident: ident.clone(),
                expected_location: self.id(),
                _phantom: PhantomData,
            },
            S::create_source(ident, Timestamped { tick: self.clone() }),
        )
    }

    pub fn cycle<S: CycleCollection<'a, TickCycleMarker, Location = Self> + DeferTick>(
        &self,
    ) -> (TickCycle<'a, S>, S)
    where
        L: NoTick,
    {
        let next_id = {
            let on_id = match self.l.id() {
                LocationId::Process(id) => id,
                LocationId::Cluster(id) => id,
                LocationId::Tick(_, _) => panic!(),
                LocationId::ExternalProcess(_) => panic!(),
            };

            let mut flow_state = self.flow_state().borrow_mut();
            let next_id_entry = flow_state.cycle_counts.entry(on_id).or_default();

            let id = *next_id_entry;
            *next_id_entry += 1;
            id
        };

        let ident = syn::Ident::new(&format!("cycle_{}", next_id), Span::call_site());

        (
            TickCycle {
                ident: ident.clone(),
                expected_location: self.id(),
                _phantom: PhantomData,
            },
            S::create_source(ident, self.clone()),
        )
    }

    pub fn cycle_with_initial<
        S: CycleCollectionWithInitial<'a, TickCycleMarker, Location = Self> + DeferTick,
    >(
        &self,
        initial: S,
    ) -> (TickCycle<'a, S>, S)
    where
        L: NoTick,
    {
        let next_id = {
            let on_id = match self.l.id() {
                LocationId::Process(id) => id,
                LocationId::Cluster(id) => id,
                LocationId::Tick(_, _) => panic!(),
                LocationId::ExternalProcess(_) => panic!(),
            };

            let mut flow_state = self.flow_state().borrow_mut();
            let next_id_entry = flow_state.cycle_counts.entry(on_id).or_default();

            let id = *next_id_entry;
            *next_id_entry += 1;
            id
        };

        let ident = syn::Ident::new(&format!("cycle_{}", next_id), Span::call_site());

        (
            TickCycle {
                ident: ident.clone(),
                expected_location: self.id(),
                _phantom: PhantomData,
            },
            S::create_source(ident, initial, self.clone()),
        )
    }
}
