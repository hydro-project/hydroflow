use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Duration;

use hydroflow::futures::stream::Stream as FuturesStream;
use hydroflow::{tokio, tokio_stream};
use proc_macro2::Span;
use stageleft::{q, Quoted};

use super::builder::FlowState;
use crate::cycle::{
    CycleCollection, CycleCollectionWithInitial, DeferTick, ForwardRef, HfCycle, TickCycle,
};
use crate::ir::{HfPlusNode, HfPlusSource};
use crate::{Bounded, HfForwardRef, Optional, Singleton, Stream, Unbounded};

pub mod external_process;
pub use external_process::ExternalProcess;

pub mod process;
pub use process::Process;

pub mod cluster;
pub use cluster::{Cluster, ClusterId};

pub mod can_send;
pub use can_send::CanSend;

pub mod tick;
pub use tick::{NoTick, Tick};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LocationId {
    Process(usize),
    Cluster(usize),
    ExternalProcess(usize),
}

impl LocationId {
    pub fn raw_id(&self) -> usize {
        match self {
            LocationId::Process(id) => *id,
            LocationId::Cluster(id) => *id,
            LocationId::ExternalProcess(id) => *id,
        }
    }
}

pub fn check_matching_location<'a, L: Location<'a>>(l1: &L, l2: &L) {
    assert_eq!(l1.id(), l2.id(), "locations do not match");
}

pub trait Location<'a>: Clone {
    fn id(&self) -> LocationId;

    fn flow_state(&self) -> &FlowState;

    fn is_top_level() -> bool;

    fn nest(&self) -> Tick<Self>
    where
        Self: NoTick,
    {
        Tick { l: self.clone() }
    }

    fn spin(&self) -> Stream<(), Unbounded, Self>
    where
        Self: Sized + NoTick,
    {
        Stream::new(
            self.clone(),
            HfPlusNode::Persist(Box::new(HfPlusNode::Source {
                source: HfPlusSource::Spin(),
                location_kind: self.id(),
            })),
        )
    }

    fn spin_batch(
        &self,
        batch_size: impl Quoted<'a, usize> + Copy + 'a,
    ) -> Stream<(), Bounded, Tick<Self>>
    where
        Self: Sized + NoTick,
    {
        self.spin()
            .flat_map(q!(move |_| 0..batch_size))
            .map(q!(|_| ()))
            .tick_batch()
    }

    fn source_stream<T, E: FuturesStream<Item = T> + Unpin>(
        &self,
        e: impl Quoted<'a, E>,
    ) -> Stream<T, Unbounded, Self>
    where
        Self: Sized + NoTick,
    {
        let e = e.splice_untyped();

        Stream::new(
            self.clone(),
            HfPlusNode::Persist(Box::new(HfPlusNode::Source {
                source: HfPlusSource::Stream(e.into()),
                location_kind: self.id(),
            })),
        )
    }

    fn source_iter<T, E: IntoIterator<Item = T>>(
        &self,
        e: impl Quoted<'a, E>,
    ) -> Stream<T, Unbounded, Self>
    where
        Self: Sized + NoTick,
    {
        // TODO(shadaj): we mark this as unbounded because we do not yet have a representation
        // for bounded top-level streams, and this is the only way to generate one
        let e = e.splice_untyped();

        Stream::new(
            self.clone(),
            HfPlusNode::Persist(Box::new(HfPlusNode::Source {
                source: HfPlusSource::Iter(e.into()),
                location_kind: self.id(),
            })),
        )
    }

    fn singleton<T: Clone>(&self, e: impl Quoted<'a, T>) -> Singleton<T, Unbounded, Self>
    where
        Self: Sized + NoTick,
    {
        // TODO(shadaj): we mark this as unbounded because we do not yet have a representation
        // for bounded top-level singletons, and this is the only way to generate one

        let e_arr = q!([e]);
        let e = e_arr.splice_untyped();

        // we do a double persist here because if the singleton shows up on every tick,
        // we first persist the source so that we store that value and then persist again
        // so that it grows every tick
        Singleton::new(
            self.clone(),
            HfPlusNode::Persist(Box::new(HfPlusNode::Persist(Box::new(
                HfPlusNode::Source {
                    source: HfPlusSource::Iter(e.into()),
                    location_kind: self.id(),
                },
            )))),
        )
    }

    fn singleton_each_tick<T: Clone>(
        &self,
        e: impl Quoted<'a, T>,
    ) -> Singleton<T, Bounded, Tick<Self>>
    where
        Self: Sized + NoTick,
    {
        self.singleton(e).latest_tick()
    }

    fn singleton_first_tick<T: Clone>(
        &self,
        e: impl Quoted<'a, T>,
    ) -> Optional<T, Bounded, Tick<Self>>
    where
        Self: Sized + NoTick,
    {
        let e_arr = q!([e]);
        let e = e_arr.splice_untyped();

        Optional::new(
            self.clone().nest(),
            HfPlusNode::Source {
                source: HfPlusSource::Iter(e.into()),
                location_kind: self.id(),
            },
        )
    }

    fn source_interval(
        &self,
        interval: impl Quoted<'a, Duration> + Copy + 'a,
    ) -> Stream<tokio::time::Instant, Unbounded, Self>
    where
        Self: Sized + NoTick,
    {
        self.source_stream(q!(tokio_stream::wrappers::IntervalStream::new(
            tokio::time::interval(interval)
        )))
    }

    fn source_interval_delayed(
        &self,
        delay: impl Quoted<'a, Duration> + Copy + 'a,
        interval: impl Quoted<'a, Duration> + Copy + 'a,
    ) -> Stream<tokio::time::Instant, Unbounded, Self>
    where
        Self: Sized + NoTick,
    {
        self.source_stream(q!(tokio_stream::wrappers::IntervalStream::new(
            tokio::time::interval_at(tokio::time::Instant::now() + delay, interval)
        )))
    }

    fn forward_ref<S: CycleCollection<'a, ForwardRef, Location = Self>>(
        &self,
    ) -> (HfForwardRef<'a, S>, S)
    where
        Self: NoTick,
    {
        let next_id = {
            let on_id = match self.id() {
                LocationId::Process(id) => id,
                LocationId::Cluster(id) => id,
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

    fn tick_forward_ref<S: CycleCollection<'a, ForwardRef, Location = Tick<Self>>>(
        &self,
    ) -> (HfForwardRef<'a, S>, S)
    where
        Self: NoTick,
    {
        let next_id = {
            let on_id = match self.id() {
                LocationId::Process(id) => id,
                LocationId::Cluster(id) => id,
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
            S::create_source(ident, self.nest().clone()),
        )
    }

    fn tick_cycle<S: CycleCollection<'a, TickCycle, Location = Tick<Self>> + DeferTick>(
        &self,
    ) -> (HfCycle<'a, S>, S)
    where
        Self: NoTick,
    {
        let next_id = {
            let on_id = match self.id() {
                LocationId::Process(id) => id,
                LocationId::Cluster(id) => id,
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
            S::create_source(ident, self.nest().clone()),
        )
    }

    fn tick_cycle_with_initial<
        S: CycleCollectionWithInitial<'a, TickCycle, Location = Tick<Self>> + DeferTick,
    >(
        &self,
        initial: S,
    ) -> (HfCycle<'a, S>, S)
    where
        Self: NoTick,
    {
        let next_id = {
            let on_id = match self.id() {
                LocationId::Process(id) => id,
                LocationId::Cluster(id) => id,
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
            S::create_source(ident, initial, self.nest().clone()),
        )
    }
}
