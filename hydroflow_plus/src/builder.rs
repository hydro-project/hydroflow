use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Duration;

use hydroflow::bytes::Bytes;
use hydroflow::futures::stream::Stream as FuturesStream;
use hydroflow::lattices::collections::MapMapValues;
use hydroflow_lang::graph::{eliminate_extra_unions_tees, HydroflowGraph};
use proc_macro2::Span;
use stageleft::*;
use syn::parse_quote;

use crate::ir::{HfPlusLeaf, HfPlusNode, HfPlusSource};
use crate::location::{
    ClusterSpec, HfSendOneToMany, HfSendOneToOne, LocalDeploy, Location, ProcessSpec,
};
use crate::stream::{Async, Windowed};
use crate::{Deploy, HfCompiled, HfCycle, RuntimeContext, Stream};

/// Tracks the leaves of the dataflow IR. This is referenced by
/// `Stream` and `HfCycle` to build the IR. The inner option will
/// be set to `None` when this builder is finalized.
pub type FlowLeaves<'a> = Rc<RefCell<Option<Vec<HfPlusLeaf<'a>>>>>;

pub struct FlowBuilder<'a, D: LocalDeploy<'a> + ?Sized> {
    ir_leaves: FlowLeaves<'a>,
    nodes: RefCell<Vec<D::Process>>,
    clusters: RefCell<Vec<D::Cluster>>,
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

impl<'a, D: LocalDeploy<'a> + ?Sized> Drop for FlowBuilder<'a, D> {
    fn drop(&mut self) {
        if !self.finalized {
            panic!("Dropped FlowBuilder without finalizing, you may have forgotten to call `with_default_optimize`, `optimize_with`, or `finalize`.");
        }
    }
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
            ir_leaves: Rc::new(RefCell::new(Some(Vec::new()))),
            nodes: RefCell::new(Vec::new()),
            clusters: RefCell::new(Vec::new()),
            cycle_ids: RefCell::new(HashMap::new()),
            next_node_id: RefCell::new(0),
            finalized: false,
            _phantom: PhantomData,
        }
    }

    pub fn finalize(mut self) -> BuiltFlow<'a, D> {
        self.finalized = true;

        BuiltFlow {
            ir: self.ir_leaves.borrow_mut().take().unwrap(),
            nodes: self.nodes.replace(vec![]),
            clusters: self.clusters.replace(vec![]),
            used: false,
            _phantom: PhantomData,
        }
    }

    pub fn with_default_optimize(self) -> BuiltFlow<'a, D> {
        self.finalize().with_default_optimize()
    }

    pub fn optimize_with(
        self,
        f: impl FnOnce(Vec<HfPlusLeaf<'a>>) -> Vec<HfPlusLeaf<'a>>,
    ) -> BuiltFlow<'a, D> {
        self.finalize().optimize_with(f)
    }

    pub fn ir_leaves(&self) -> &FlowLeaves<'a> {
        &self.ir_leaves
    }

    pub fn process(&self, spec: &impl ProcessSpec<'a, D>) -> D::Process {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        let node = spec.build(id);
        self.nodes.borrow_mut().push(node.clone());

        node
    }

    pub fn cluster(&self, spec: &impl ClusterSpec<'a, D>) -> D::Cluster {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        let cluster = spec.build(id);
        self.clusters.borrow_mut().push(cluster.clone());

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

    pub fn source_external<L>(&self, on: &L) -> (L::Port, Stream<'a, Bytes, Async, L>)
    where
        L: Location + Clone + HfSendOneToOne<L>,
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
}

pub struct BuiltFlow<'a, D: LocalDeploy<'a>> {
    pub(crate) ir: Vec<HfPlusLeaf<'a>>,
    nodes: Vec<D::Process>,
    clusters: Vec<D::Cluster>,
    used: bool,

    _phantom: PhantomData<&'a mut &'a D>,
}

impl<'a, D: LocalDeploy<'a>> Drop for BuiltFlow<'a, D> {
    fn drop(&mut self) {
        if !self.used {
            panic!("Dropped BuiltFlow without instantiating, you may have forgotten to call `compile` or `deploy`.");
        }
    }
}

impl<'a, D: LocalDeploy<'a>> BuiltFlow<'a, D> {
    pub fn ir(&self) -> &Vec<HfPlusLeaf<'a>> {
        &self.ir
    }

    pub fn optimize_with(
        mut self,
        f: impl FnOnce(Vec<HfPlusLeaf<'a>>) -> Vec<HfPlusLeaf<'a>>,
    ) -> BuiltFlow<'a, D> {
        self.used = true;
        BuiltFlow {
            ir: f(std::mem::take(&mut self.ir)),
            nodes: std::mem::take(&mut self.nodes),
            clusters: std::mem::take(&mut self.clusters),
            used: false,
            _phantom: PhantomData,
        }
    }
}

fn build_inner(ir: Vec<HfPlusLeaf>) -> BTreeMap<usize, HydroflowGraph> {
    let mut builders = BTreeMap::new();
    let mut built_tees = HashMap::new();
    let mut next_stmt_id = 0;
    for leaf in ir {
        leaf.emit(&mut builders, &mut built_tees, &mut next_stmt_id);
    }

    builders.map_values(|v| {
        let (mut flat_graph, _, _) = v.build();
        eliminate_extra_unions_tees(&mut flat_graph);
        flat_graph
    })
}

impl<'a, D: LocalDeploy<'a>> BuiltFlow<'a, D> {
    pub fn compile(mut self) -> HfCompiled<'a, D::GraphId> {
        self.used = true;

        let mut seen_tees: HashMap<_, _> = HashMap::new();
        let ir_leaves_networked: Vec<HfPlusLeaf> = std::mem::take(&mut self.ir)
            .into_iter()
            .map(|leaf| leaf.instantiate_network(&mut seen_tees))
            .collect();

        HfCompiled {
            hydroflow_ir: build_inner(ir_leaves_networked),
            _phantom: PhantomData,
        }
    }

    pub fn with_default_optimize(self) -> BuiltFlow<'a, D> {
        self.optimize_with(super::persist_pullup::persist_pullup)
    }
}

impl<'a, D: Deploy<'a>> BuiltFlow<'a, D> {
    #[must_use]
    pub fn deploy(mut self, env: &mut D::InstantiateEnv) -> (Vec<D::Process>, Vec<D::Cluster>) {
        self.used = true;

        let mut seen_tees_instantiate: HashMap<_, _> = HashMap::new();
        let ir_leaves_networked: Vec<HfPlusLeaf> = std::mem::take(&mut self.ir)
            .into_iter()
            .map(|leaf| leaf.instantiate_network(&mut seen_tees_instantiate))
            .collect();

        let mut compiled = build_inner(ir_leaves_networked.clone());
        let mut meta = D::Meta::default();

        let (mut nodes, mut clusters): (Vec<D::Process>, Vec<D::Cluster>) = (
            std::mem::take(&mut self.nodes)
                .into_iter()
                .map(|node| {
                    node.instantiate(env, &mut meta, compiled.remove(&node.id()).unwrap());
                    node
                })
                .collect(),
            std::mem::take(&mut self.clusters)
                .into_iter()
                .map(|cluster| {
                    cluster.instantiate(env, &mut meta, compiled.remove(&cluster.id()).unwrap());
                    cluster
                })
                .collect(),
        );

        for node in &mut nodes {
            node.update_meta(&meta);
        }

        for cluster in &mut clusters {
            cluster.update_meta(&meta);
        }

        let mut seen_tees_connect = HashMap::new();
        for leaf in ir_leaves_networked {
            leaf.connect_network(&mut seen_tees_connect);
        }

        (nodes, clusters)
    }
}
