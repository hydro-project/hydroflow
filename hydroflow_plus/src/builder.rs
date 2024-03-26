use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Duration;

use hydroflow::bytes::Bytes;
use hydroflow::futures::stream::Stream as FuturesStream;
use hydroflow::lattices::collections::MapMapValues;
use hydroflow_lang::graph::eliminate_extra_unions_tees;
use proc_macro2::Span;
use stageleft::*;
use syn::parse_quote;

use crate::ir::{HfPlusLeaf, HfPlusNode, HfPlusSource};
use crate::location::{
    ClusterSpec, HfSendOneToMany, HfSendOneToOne, LocalDeploy, Location, ProcessSpec,
};
use crate::stream::{Async, Windowed};
use crate::{HfCompiled, HfCycle, RuntimeContext, Stream};

pub struct FlowBuilder<'a, D: LocalDeploy<'a> + ?Sized> {
    ir_leaves: Rc<RefCell<Vec<HfPlusLeaf>>>,
    nodes: RefCell<Vec<D::Process>>,
    clusters: RefCell<Vec<D::Cluster>>,
    cycle_ids: RefCell<HashMap<usize, usize>>,

    /// Tracks metadata about concrete deployments in this graph, such
    /// as the IDs of each node in a cluster. This is written to
    /// by `ProcessSpec` and `ClusterSpec` and is written to
    /// each instantiated node and cluster via `HfNode::update_meta`.
    meta: RefCell<D::Meta>,

    next_node_id: RefCell<usize>,

    /// 'a on a FlowBuilder is used to ensure that staged code does not
    /// capture more data that it is allowed to; 'a is generated at the
    /// entrypoint of the staged code and we keep it invariant here
    /// to enforce the appropriate constraints
    _phantom: PhantomData<&'a mut &'a ()>,
}

impl<'a, D: LocalDeploy<'a>> QuotedContext for FlowBuilder<'a, D> {
    fn create() -> Self {
        FlowBuilder::new()
    }
}

impl<'a, D: LocalDeploy<'a>> FlowBuilder<'a, D> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> FlowBuilder<'a, D> {
        FlowBuilder {
            ir_leaves: Rc::new(RefCell::new(Vec::new())),
            nodes: RefCell::new(Vec::new()),
            clusters: RefCell::new(Vec::new()),
            cycle_ids: RefCell::new(HashMap::new()),
            meta: RefCell::new(Default::default()),
            next_node_id: RefCell::new(0),
            _phantom: PhantomData,
        }
    }

    pub fn extract(self) -> BuiltFlow<'a, D> {
        BuiltFlow {
            ir: self.ir_leaves.borrow().clone(),
            nodes: self.nodes.into_inner(),
            clusters: self.clusters.into_inner(),
            _phantom: PhantomData,
        }
    }

    pub fn ir_leaves(&self) -> &Rc<RefCell<Vec<HfPlusLeaf>>> {
        &self.ir_leaves
    }

    pub fn process(&self, spec: &impl ProcessSpec<'a, D>) -> D::Process {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        let node = spec.build(id, &mut self.meta.borrow_mut());
        self.nodes.borrow_mut().push(node.clone());

        self.update_metas();

        node
    }

    pub fn cluster(&self, spec: &impl ClusterSpec<'a, D>) -> D::Cluster {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        let cluster = spec.build(id, &mut self.meta.borrow_mut());
        self.clusters.borrow_mut().push(cluster.clone());

        self.update_metas();

        cluster
    }

    pub fn runtime_context(&self) -> RuntimeContext<'a> {
        RuntimeContext {
            _phantom: PhantomData,
        }
    }

    pub fn spin<L: Location + Clone>(&self, on: &L) -> Stream<'a, (), Async, L> {
        Stream::new(
            on.clone(),
            self.ir_leaves().clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Spin(),
                location_id: on.id(),
            },
        )
    }

    pub fn spin_batch<L: Location + Clone>(
        &self,
        on: &L,
        batch_size: impl Quoted<'a, usize> + Copy + 'a,
    ) -> Stream<'a, (), Windowed, L> {
        self.spin(on)
            .flat_map(q!(move |_| 0..batch_size))
            .map(q!(|_| ()))
            .tick_batch()
    }

    pub fn source_stream<T, E: FuturesStream<Item = T> + Unpin, L: Location + Clone>(
        &self,
        on: &L,
        e: impl Quoted<'a, E>,
    ) -> Stream<'a, T, Async, L> {
        let e = e.splice();

        Stream::new(
            on.clone(),
            self.ir_leaves().clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Stream(e.into()),
                location_id: on.id(),
            },
        )
    }

    pub fn source_external<L: Location + Clone>(
        &self,
        on: &L,
    ) -> (L::Port, Stream<'a, Bytes, Async, L>)
    where
        L: HfSendOneToOne<L>,
    {
        let port = on.next_port();
        let source_pipeline = L::gen_source_statement(on, &port);

        let process: syn::Expr = parse_quote!(|b| b.unwrap().freeze());

        (
            port,
            Stream::new(
                on.clone(),
                self.ir_leaves().clone(),
                HfPlusNode::Map {
                    f: process.into(),
                    input: Box::new(HfPlusNode::Source {
                        source: HfPlusSource::Stream(source_pipeline.into()),
                        location_id: on.id(),
                    }),
                },
            ),
        )
    }

    pub fn many_source_external<S, Cid, L: Location + Clone>(
        &self,
        on: &L,
    ) -> (L::Port, Stream<'a, Bytes, Async, L>)
    where
        S: Location + HfSendOneToMany<L, Cid>,
    {
        let port = on.next_port();
        let source_pipeline = S::gen_source_statement(on, &port);

        let process: syn::Expr = parse_quote!(|b| b.unwrap().freeze());

        (
            port,
            Stream::new(
                on.clone(),
                self.ir_leaves().clone(),
                HfPlusNode::Map {
                    f: process.into(),
                    input: Box::new(HfPlusNode::Source {
                        source: HfPlusSource::Stream(source_pipeline.into()),
                        location_id: on.id(),
                    }),
                },
            ),
        )
    }

    pub fn source_iter<T, E: IntoIterator<Item = T>, L: Location + Clone>(
        &self,
        on: &L,
        e: impl Quoted<'a, E>,
    ) -> Stream<'a, T, Windowed, L> {
        let e = e.splice();

        Stream::new(
            on.clone(),
            self.ir_leaves().clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Iter(e.into()),
                location_id: on.id(),
            },
        )
    }

    pub fn source_interval<L: Location + Clone>(
        &self,
        on: &L,
        interval: impl Quoted<'a, Duration> + Copy + 'a,
    ) -> Stream<'a, hydroflow::tokio::time::Instant, Async, L> {
        let interval = interval.splice();

        Stream::new(
            on.clone(),
            self.ir_leaves().clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Interval(interval.into()),
                location_id: on.id(),
            },
        )
    }

    pub fn cycle<T, W, L: Location + Clone>(
        &self,
        on: &L,
    ) -> (HfCycle<'a, T, W, L>, Stream<'a, T, W, L>) {
        let next_id = {
            let mut cycle_ids = self.cycle_ids.borrow_mut();
            let next_id_entry = cycle_ids.entry(on.id()).or_default();

            let id = *next_id_entry;
            *next_id_entry += 1;
            id
        };

        let ident = syn::Ident::new(&format!("cycle_{}", next_id), Span::call_site());

        (
            HfCycle {
                ident: ident.clone(),
                node: on.clone(),
                ir_leaves: self.ir_leaves().clone(),
                _phantom: PhantomData,
            },
            Stream::new(
                on.clone(),
                self.ir_leaves().clone(),
                HfPlusNode::CycleSource {
                    ident,
                    location_id: on.id(),
                },
            ),
        )
    }

    fn update_metas(&self) {
        self.nodes
            .borrow_mut()
            .iter_mut()
            .for_each(|n| n.update_meta(&self.meta.borrow()));
        self.clusters
            .borrow_mut()
            .iter_mut()
            .for_each(|n| n.update_meta(&self.meta.borrow()));
    }
}

#[derive(Clone)]
pub struct BuiltFlow<'a, D: LocalDeploy<'a>> {
    pub(crate) ir: Vec<HfPlusLeaf>,
    nodes: Vec<D::Process>,
    clusters: Vec<D::Cluster>,

    _phantom: PhantomData<&'a mut &'a D>,
}

impl<'a, D: LocalDeploy<'a>> BuiltFlow<'a, D> {
    pub fn ir(&self) -> &Vec<HfPlusLeaf> {
        &self.ir
    }

    pub fn optimize_with(
        self,
        f: impl FnOnce(Vec<HfPlusLeaf>) -> Vec<HfPlusLeaf>,
    ) -> BuiltFlow<'a, D> {
        BuiltFlow {
            ir: f(self.ir),
            nodes: self.nodes,
            clusters: self.clusters,
            _phantom: PhantomData,
        }
    }
}

fn build_inner<'a, D: LocalDeploy<'a>>(me: BuiltFlow<'a, D>) -> HfCompiled<'a, D::GraphId> {
    let mut builders = BTreeMap::new();
    let mut built_tees = HashMap::new();
    let mut next_stmt_id = 0;
    for leaf in me.ir {
        leaf.emit(&mut builders, &mut built_tees, &mut next_stmt_id);
    }

    HfCompiled {
        hydroflow_ir: builders.map_values(|v| {
            let (mut flat_graph, _, _) = v.build();
            eliminate_extra_unions_tees(&mut flat_graph);
            flat_graph
        }),
        _phantom: PhantomData,
    }
}

impl<'a, D: LocalDeploy<'a>> BuiltFlow<'a, D> {
    pub fn no_optimize(self) -> HfCompiled<'a, D::GraphId> {
        build_inner(self)
    }

    pub fn optimize_default(self) -> HfCompiled<'a, D::GraphId> {
        self.optimize_with(super::persist_pullup::persist_pullup)
            .no_optimize()
    }
}
