use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Duration;

use hydroflow::futures::stream::Stream as FuturesStream;
use internal::TokenStream;
use proc_macro2::Span;
use quote::quote;
use runtime_support::FreeVariable;
use stageleft::*;

use crate::ir::{HfPlusLeaf, HfPlusNode, HfPlusSource};
use crate::location::{Cluster, Location, LocationId, Process};
use crate::stream::{Async, Windowed};
use crate::{HfCycle, RuntimeContext, Stream};

pub mod built;
pub mod deploy;

/// Tracks the leaves of the dataflow IR. This is referenced by
/// `Stream` and `HfCycle` to build the IR. The inner option will
/// be set to `None` when this builder is finalized.
pub type FlowLeaves<'a> = Rc<RefCell<Option<Vec<HfPlusLeaf<'a>>>>>;

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
struct ClusterSelfId<'a> {
    id: usize,
    _phantom: PhantomData<&'a mut &'a u32>,
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
    #[allow(clippy::new_without_default)]
    pub fn new() -> FlowBuilder<'a> {
        FlowBuilder {
            ir_leaves: Rc::new(RefCell::new(Some(Vec::new()))),
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
            ir: self.ir_leaves.borrow_mut().take().unwrap(),
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

    pub fn cluster_members<C>(
        &self,
        cluster: &Cluster<C>,
    ) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a {
        ClusterIds {
            id: cluster.id,
            _phantom: PhantomData,
        }
    }

    pub fn cluster_self_id<C>(&self, cluster: &Cluster<C>) -> impl Quoted<'a, u32> + Copy + 'a {
        ClusterSelfId {
            id: cluster.id,
            _phantom: PhantomData,
        }
    }

    pub fn spin<L: Location>(&self, on: &L) -> Stream<'a, (), Async, L> {
        Stream::new(
            on.id(),
            self.ir_leaves().clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Spin(),
                location_kind: on.id(),
            },
        )
    }

    pub fn spin_batch<L: Location>(
        &self,
        on: &L,
        batch_size: impl Quoted<'a, usize> + Copy + 'a,
    ) -> Stream<'a, (), Windowed, L> {
        self.spin(on)
            .flat_map(q!(move |_| 0..batch_size))
            .map(q!(|_| ()))
            .tick_batch()
    }

    pub fn source_stream<T, E: FuturesStream<Item = T> + Unpin, L: Location>(
        &self,
        on: &L,
        e: impl Quoted<'a, E>,
    ) -> Stream<'a, T, Async, L> {
        let e = e.splice();

        Stream::new(
            on.id(),
            self.ir_leaves().clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Stream(e.into()),
                location_kind: on.id(),
            },
        )
    }

    pub fn source_iter<T, E: IntoIterator<Item = T>, L: Location>(
        &self,
        on: &L,
        e: impl Quoted<'a, E>,
    ) -> Stream<'a, T, Windowed, L> {
        let e = e.splice();

        Stream::new(
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
    ) -> Stream<'a, hydroflow::tokio::time::Instant, Async, L> {
        let interval = interval.splice();

        Stream::new(
            on.id(),
            self.ir_leaves().clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Interval(interval.into()),
                location_kind: on.id(),
            },
        )
    }

    pub fn cycle<T, W, L: Location>(&self, on: &L) -> (HfCycle<'a, T, W, L>, Stream<'a, T, W, L>) {
        let next_id = {
            let on_id = match on.id() {
                LocationId::Process(id) => id,
                LocationId::Cluster(id) => id,
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
                location_kind: on.id(),
                ir_leaves: self.ir_leaves().clone(),
                _phantom: PhantomData,
            },
            Stream::new(
                on.id(),
                self.ir_leaves().clone(),
                HfPlusNode::CycleSource {
                    ident,
                    location_kind: on.id(),
                },
            ),
        )
    }
}
