use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Duration;

use hydroflow::bytes::Bytes;
use hydroflow::futures::stream::Stream as FuturesStream;
use hydroflow::{tokio, tokio_stream};
use internal::TokenStream;
use proc_macro2::Span;
use quote::quote;
use runtime_support::FreeVariable;
use serde::de::DeserializeOwned;
use serde::Serialize;
use stageleft::*;

use crate::cycle::{CycleCollection, CycleCollectionWithInitial};
use crate::ir::{HfPlusLeaf, HfPlusNode, HfPlusSource};
use crate::location::{
    Cluster, ExternalBincodeSink, ExternalBytesPort, ExternalProcess, Location, LocationId, Process,
};
use crate::stream::{Bounded, NoTick, Tick, Unbounded};
use crate::{HfCycle, Optional, RuntimeContext, Singleton, Stream};

pub mod built;
pub mod deploy;

/// Tracks the leaves of the dataflow IR. This is referenced by
/// `Stream` and `HfCycle` to build the IR. The inner option will
/// be set to `None` when this builder is finalized.
///
/// The second `usize` is used to generate unique identifiers for external
/// outputs of the dataflow.
pub type FlowLeaves<'a> = Rc<RefCell<(Option<Vec<HfPlusLeaf<'a>>>, usize)>>;

pub type ExternalPortCounter = Rc<RefCell<usize>>;

#[derive(Copy, Clone)]
pub struct ClusterIds<'a> {
    pub(crate) id: usize,
    pub(crate) _phantom: PhantomData<&'a mut &'a Vec<u32>>,
}

impl<'a> FreeVariable<&'a Vec<u32>> for ClusterIds<'a> {
    fn to_tokens(self) -> (Option<TokenStream>, Option<TokenStream>)
    where
        Self: Sized,
    {
        let ident = syn::Ident::new(
            &format!("__hydroflow_plus_cluster_ids_{}", self.id),
            Span::call_site(),
        );
        (None, Some(quote! { #ident }))
    }
}

impl<'a> Quoted<'a, &'a Vec<u32>> for ClusterIds<'a> {}

#[derive(Copy, Clone)]
pub(crate) struct ClusterSelfId<'a> {
    pub(crate) id: usize,
    pub(crate) _phantom: PhantomData<&'a mut &'a u32>,
}

impl<'a> FreeVariable<u32> for ClusterSelfId<'a> {
    fn to_tokens(self) -> (Option<TokenStream>, Option<TokenStream>)
    where
        Self: Sized,
    {
        let ident = syn::Ident::new(
            &format!("__hydroflow_plus_cluster_self_id_{}", self.id),
            Span::call_site(),
        );
        (None, Some(quote! { #ident }))
    }
}

impl<'a> Quoted<'a, u32> for ClusterSelfId<'a> {}

pub struct FlowBuilder<'a> {
    ir_leaves: FlowLeaves<'a>,
    nodes: RefCell<Vec<usize>>,
    clusters: RefCell<Vec<usize>>,
    cycle_ids: RefCell<HashMap<usize, usize>>,

    next_node_id: RefCell<usize>,

    /// Tracks whether this flow has been finalized; it is an error to
    /// drop without finalizing.
    finalized: bool,

    /// 'a on a FlowBuilder is used to ensure that staged code does not
    /// capture more data that it is allowed to; 'a is generated at the
    /// entrypoint of the staged code and we keep it invariant here
    /// to enforce the appropriate constraints
    _phantom: PhantomData<&'a mut &'a ()>,
}

impl<'a> Drop for FlowBuilder<'a> {
    fn drop(&mut self) {
        if !self.finalized {
            panic!("Dropped FlowBuilder without finalizing, you may have forgotten to call `with_default_optimize`, `optimize_with`, or `finalize`.");
        }
    }
}

impl<'a> QuotedContext for FlowBuilder<'a> {
    fn create() -> Self {
        FlowBuilder::new()
    }
}

impl<'a> FlowBuilder<'a> {
    #[expect(
        clippy::new_without_default,
        reason = "call `new` explicitly, not `default`"
    )]
    pub fn new() -> FlowBuilder<'a> {
        FlowBuilder {
            ir_leaves: Rc::new(RefCell::new((Some(Vec::new()), 0))),
            nodes: RefCell::new(vec![]),
            clusters: RefCell::new(vec![]),
            cycle_ids: RefCell::new(HashMap::new()),
            next_node_id: RefCell::new(0),
            finalized: false,
            _phantom: PhantomData,
        }
    }

    pub fn finalize(mut self) -> built::BuiltFlow<'a> {
        self.finalized = true;

        built::BuiltFlow {
            ir: self.ir_leaves.borrow_mut().0.take().unwrap(),
            processes: self.nodes.replace(vec![]),
            clusters: self.clusters.replace(vec![]),
            used: false,
            _phantom: PhantomData,
        }
    }

    pub fn with_default_optimize(self) -> built::BuiltFlow<'a> {
        self.finalize().with_default_optimize()
    }

    pub fn optimize_with(
        self,
        f: impl FnOnce(Vec<HfPlusLeaf<'a>>) -> Vec<HfPlusLeaf<'a>>,
    ) -> built::BuiltFlow<'a> {
        self.finalize().optimize_with(f)
    }

    pub fn ir_leaves(&self) -> &FlowLeaves<'a> {
        &self.ir_leaves
    }

    pub fn process<P>(&self) -> Process<P> {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        self.nodes.borrow_mut().push(id);

        Process {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn external_process<P>(&self) -> ExternalProcess<P> {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        self.nodes.borrow_mut().push(id);

        ExternalProcess {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn cluster<C>(&self) -> Cluster<C> {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        self.clusters.borrow_mut().push(id);

        Cluster {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn runtime_context(&self) -> RuntimeContext<'a> {
        RuntimeContext {
            _phantom: PhantomData,
        }
    }

    pub fn spin<L: Location>(&self, on: &L) -> Stream<'a, (), Unbounded, NoTick, L> {
        Stream::new(
            on.id(),
            self.ir_leaves().clone(),
            HfPlusNode::Persist(Box::new(HfPlusNode::Source {
                source: HfPlusSource::Spin(),
                location_kind: on.id(),
            })),
        )
    }

    pub fn spin_batch<L: Location>(
        &self,
        on: &L,
        batch_size: impl Quoted<'a, usize> + Copy + 'a,
    ) -> Stream<'a, (), Bounded, Tick, L> {
        self.spin(on)
            .flat_map(q!(move |_| 0..batch_size))
            .map(q!(|_| ()))
            .tick_batch()
    }

    pub fn source_external_bytes<P, L: Location>(
        &self,
        from: &ExternalProcess<P>,
        to: &L,
    ) -> (ExternalBytesPort, Stream<'a, Bytes, Unbounded, NoTick, L>) {
        let next_external_port_id = {
            let mut ir_leaves = self.ir_leaves.borrow_mut();
            let id = ir_leaves.1;
            ir_leaves.1 += 1;
            id
        };

        (
            ExternalBytesPort {
                process_id: from.id,
                port_id: next_external_port_id,
            },
            Stream::new(
                to.id(),
                self.ir_leaves().clone(),
                HfPlusNode::Persist(Box::new(HfPlusNode::Network {
                    from_location: LocationId::ExternalProcess(from.id),
                    from_key: Some(next_external_port_id),
                    to_location: to.id(),
                    to_key: None,
                    serialize_pipeline: None,
                    instantiate_fn: crate::ir::DebugInstantiate::Building(),
                    deserialize_pipeline: Some(syn::parse_quote!(map(|b| b.unwrap().freeze()))),
                    input: Box::new(HfPlusNode::Source {
                        source: HfPlusSource::ExternalNetwork(),
                        location_kind: LocationId::ExternalProcess(from.id),
                    }),
                })),
            ),
        )
    }

    pub fn source_external_bincode<P, L: Location, T: Serialize + DeserializeOwned>(
        &self,
        from: &ExternalProcess<P>,
        to: &L,
    ) -> (ExternalBincodeSink<T>, Stream<'a, T, Unbounded, NoTick, L>) {
        let next_external_port_id = {
            let mut ir_leaves = self.ir_leaves.borrow_mut();
            let id = ir_leaves.1;
            ir_leaves.1 += 1;
            id
        };

        (
            ExternalBincodeSink {
                process_id: from.id,
                port_id: next_external_port_id,
                _phantom: PhantomData,
            },
            Stream::new(
                to.id(),
                self.ir_leaves().clone(),
                HfPlusNode::Persist(Box::new(HfPlusNode::Network {
                    from_location: LocationId::ExternalProcess(from.id),
                    from_key: Some(next_external_port_id),
                    to_location: to.id(),
                    to_key: None,
                    serialize_pipeline: None,
                    instantiate_fn: crate::ir::DebugInstantiate::Building(),
                    deserialize_pipeline: Some(crate::stream::deserialize_bincode::<T>(false)),
                    input: Box::new(HfPlusNode::Source {
                        source: HfPlusSource::ExternalNetwork(),
                        location_kind: LocationId::ExternalProcess(from.id),
                    }),
                })),
            ),
        )
    }

    pub fn source_stream<T, E: FuturesStream<Item = T> + Unpin, L: Location>(
        &self,
        on: &L,
        e: impl Quoted<'a, E>,
    ) -> Stream<'a, T, Unbounded, NoTick, L> {
        let e = e.splice_untyped();

        Stream::new(
            on.id(),
            self.ir_leaves().clone(),
            HfPlusNode::Persist(Box::new(HfPlusNode::Source {
                source: HfPlusSource::Stream(e.into()),
                location_kind: on.id(),
            })),
        )
    }

    pub fn source_iter<T, E: IntoIterator<Item = T>, L: Location>(
        &self,
        on: &L,
        e: impl Quoted<'a, E>,
    ) -> Stream<'a, T, Bounded, NoTick, L> {
        let e = e.splice_untyped();

        Stream::new(
            on.id(),
            self.ir_leaves().clone(),
            HfPlusNode::Persist(Box::new(HfPlusNode::Source {
                source: HfPlusSource::Iter(e.into()),
                location_kind: on.id(),
            })),
        )
    }

    pub fn singleton<T: Clone, L: Location>(
        &self,
        on: &L,
        e: impl Quoted<'a, T>,
    ) -> Singleton<'a, T, Bounded, NoTick, L> {
        let e_arr = q!([e]);
        let e = e_arr.splice_untyped();

        // we do a double persist here because if the singleton shows up on every tick,
        // we first persist the source so that we store that value and then persist again
        // so that it grows every tick
        Singleton::new(
            on.id(),
            self.ir_leaves().clone(),
            HfPlusNode::Persist(Box::new(HfPlusNode::Persist(Box::new(
                HfPlusNode::Source {
                    source: HfPlusSource::Iter(e.into()),
                    location_kind: on.id(),
                },
            )))),
        )
    }

    pub fn singleton_first_tick<T: Clone, L: Location>(
        &self,
        on: &L,
        e: impl Quoted<'a, T>,
    ) -> Optional<'a, T, Bounded, Tick, L> {
        let e_arr = q!([e]);
        let e = e_arr.splice_untyped();

        Optional::new(
            on.id(),
            self.ir_leaves().clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Iter(e.into()),
                location_kind: on.id(),
            },
        )
    }

    pub fn source_interval<L: Location>(
        &self,
        on: &L,
        interval: impl Quoted<'a, Duration> + Copy + 'a,
    ) -> Optional<'a, (), Unbounded, NoTick, L> {
        let interval = interval.splice_untyped();

        Optional::new(
            on.id(),
            self.ir_leaves().clone(),
            HfPlusNode::Persist(Box::new(HfPlusNode::Source {
                source: HfPlusSource::Interval(interval.into()),
                location_kind: on.id(),
            })),
        )
    }

    pub fn source_interval_delayed<L: Location>(
        &self,
        on: &L,
        delay: impl Quoted<'a, Duration> + Copy + 'a,
        interval: impl Quoted<'a, Duration> + Copy + 'a,
    ) -> Optional<'a, tokio::time::Instant, Unbounded, NoTick, L> {
        self.source_stream(
            on,
            q!(tokio_stream::wrappers::IntervalStream::new(
                tokio::time::interval_at(tokio::time::Instant::now() + delay, interval)
            )),
        )
        .tick_batch()
        .first()
        .latest()
    }

    pub fn tick_cycle<S: CycleCollection<'a, Tick>>(
        &self,
        on: &S::Location,
    ) -> (HfCycle<'a, Tick, S>, S) {
        let next_id = {
            let on_id = match on.id() {
                LocationId::Process(id) => id,
                LocationId::Cluster(id) => id,
                LocationId::ExternalProcess(_) => panic!(),
            };

            let mut cycle_ids = self.cycle_ids.borrow_mut();
            let next_id_entry = cycle_ids.entry(on_id).or_default();

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
            S::create_source(ident, self.ir_leaves.clone(), on.id()),
        )
    }

    pub fn cycle<S: CycleCollection<'a, NoTick>>(
        &self,
        on: &S::Location,
    ) -> (HfCycle<'a, NoTick, S>, S) {
        let next_id = {
            let on_id = match on.id() {
                LocationId::Process(id) => id,
                LocationId::Cluster(id) => id,
                LocationId::ExternalProcess(_) => panic!(),
            };

            let mut cycle_ids = self.cycle_ids.borrow_mut();
            let next_id_entry = cycle_ids.entry(on_id).or_default();

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
            S::create_source(ident, self.ir_leaves.clone(), on.id()),
        )
    }

    pub fn tick_cycle_with_initial<S: CycleCollectionWithInitial<'a, Tick>>(
        &self,
        on: &S::Location,
        initial: S,
    ) -> (HfCycle<'a, Tick, S>, S) {
        let next_id = {
            let on_id = match on.id() {
                LocationId::Process(id) => id,
                LocationId::Cluster(id) => id,
                LocationId::ExternalProcess(_) => panic!(),
            };

            let mut cycle_ids = self.cycle_ids.borrow_mut();
            let next_id_entry = cycle_ids.entry(on_id).or_default();

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
            S::create_source(ident, self.ir_leaves.clone(), initial, on.id()),
        )
    }
}
