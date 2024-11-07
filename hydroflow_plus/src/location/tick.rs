use std::marker::PhantomData;

use proc_macro2::Span;
use stageleft::{q, Quoted};

use super::{Cluster, Location, LocationId, Process};
use crate::builder::FlowState;
use crate::cycle::{
    CycleCollection, CycleCollectionWithInitial, DeferTick, ForwardRef, HfCycle, HfForwardRef,
    TickCycle,
};
use crate::ir::{HfPlusNode, HfPlusSource};
use crate::{Bounded, Optional, Singleton, Stream};

pub trait NoTick {}
impl<T> NoTick for Process<'_, T> {}
impl<T> NoTick for Cluster<'_, T> {}

/// Marks the stream as being inside the single global clock domain.
#[derive(Clone)]
pub struct Tick<L> {
    pub(crate) id: usize,
    pub(crate) l: L,
}

impl<'a, L: Location<'a>> Tick<L> {
    pub fn outer(&self) -> &L {
        &self.l
    }

    pub fn spin_batch(
        &self,
        batch_size: impl Quoted<'a, usize> + Copy + 'a,
    ) -> Stream<(), Bounded, Self>
    where
        L: NoTick,
    {
        self.l
            .spin()
            .flat_map(q!(move |_| 0..batch_size))
            .map(q!(|_| ()))
            .tick_batch(self)
    }

    pub fn singleton<T: Clone>(&self, e: impl Quoted<'a, T>) -> Singleton<T, Bounded, Self>
    where
        L: NoTick,
    {
        self.outer().singleton(e).latest_tick(self)
    }

    pub fn singleton_first_tick<T: Clone>(
        &self,
        e: impl Quoted<'a, T>,
    ) -> Optional<T, Bounded, Self>
    where
        L: NoTick,
    {
        let e_arr = q!([e]);
        let e = e_arr.splice_untyped();

        Optional::new(
            self.clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Iter(e.into()),
                location_kind: self.l.id(),
            },
        )
    }

    pub fn forward_ref<S: CycleCollection<'a, ForwardRef, Location = Self>>(
        &self,
    ) -> (HfForwardRef<'a, S>, S)
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
            HfForwardRef {
                ident: ident.clone(),
                _phantom: PhantomData,
            },
            S::create_source(ident, self.clone()),
        )
    }

    pub fn cycle<S: CycleCollection<'a, TickCycle, Location = Self> + DeferTick>(
        &self,
    ) -> (HfCycle<'a, S>, S)
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
            HfCycle {
                ident: ident.clone(),
                _phantom: PhantomData,
            },
            S::create_source(ident, self.clone()),
        )
    }

    pub fn cycle_with_initial<
        S: CycleCollectionWithInitial<'a, TickCycle, Location = Self> + DeferTick,
    >(
        &self,
        initial: S,
    ) -> (HfCycle<'a, S>, S)
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
            HfCycle {
                ident: ident.clone(),
                _phantom: PhantomData,
            },
            S::create_source(ident, initial, self.clone()),
        )
    }
}

impl<'a, L: Location<'a>> Location<'a> for Tick<L> {
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
