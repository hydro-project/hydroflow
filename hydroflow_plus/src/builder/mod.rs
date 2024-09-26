use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use internal::TokenStream;
use proc_macro2::Span;
use quote::quote;
use runtime_support::FreeVariable;
use stageleft::*;

use crate::cycle::{CycleCollection, CycleCollectionWithInitial};
use crate::ir::HfPlusLeaf;
use crate::location::{Cluster, ExternalProcess, Location, LocationId, Process};
use crate::stream::{NoTick, Tick};
use crate::{HfCycle, RuntimeContext};

pub mod built;
pub mod deploy;

/// Tracks the leaves of the dataflow IR. This is referenced by
/// `Stream` and `HfCycle` to build the IR. The inner option will
/// be set to `None` when this builder is finalized.
///
/// The second `usize` is used to generate unique identifiers for external
/// outputs of the dataflow.
pub type FlowLeaves = Rc<RefCell<(Option<Vec<HfPlusLeaf>>, usize)>>;

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
    ir_leaves: FlowLeaves,
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
        f: impl FnOnce(Vec<HfPlusLeaf>) -> Vec<HfPlusLeaf>,
    ) -> built::BuiltFlow<'a> {
        self.finalize().optimize_with(f)
    }

    pub fn ir_leaves(&self) -> &FlowLeaves {
        &self.ir_leaves
    }

    pub fn process<P>(&self) -> Process<'a, P> {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        self.nodes.borrow_mut().push(id);

        Process {
            id,
            ir_leaves: self.ir_leaves().clone(),
            _phantom: PhantomData,
        }
    }

    pub fn external_process<P>(&self) -> ExternalProcess<'a, P> {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        self.nodes.borrow_mut().push(id);

        ExternalProcess {
            id,
            ir_leaves: self.ir_leaves().clone(),
            _phantom: PhantomData,
        }
    }

    pub fn cluster<C>(&self) -> Cluster<'a, C> {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        self.clusters.borrow_mut().push(id);

        Cluster {
            id,
            ir_leaves: self.ir_leaves().clone(),
            _phantom: PhantomData,
        }
    }

    pub fn runtime_context(&self) -> RuntimeContext<'a> {
        RuntimeContext {
            _phantom: PhantomData,
        }
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
