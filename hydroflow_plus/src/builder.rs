use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Duration;

use hydroflow::futures::stream::Stream as FuturesStream;
use hydroflow::lattices::collections::MapMapValues;
use hydroflow_lang::graph::{eliminate_extra_unions_tees, HydroflowGraph};
use proc_macro2::Span;
use stageleft::*;

use crate::deploy::{ClusterSpec, Deploy, LocalDeploy, Node, ProcessSpec};
use crate::ir::{HfPlusLeaf, HfPlusNode, HfPlusSource};
use crate::location::{Cluster, Location, LocationKind, Process};
use crate::stream::{Async, Windowed};
use crate::{HfCompiled, HfCycle, RuntimeContext, Stream};

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

    pub fn process<P>(&self, spec: impl ProcessSpec<'a, D>) -> Process<P> {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        let node = spec.build(id);
        self.nodes.borrow_mut().push(node);

        Process {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn cluster<C>(&self, spec: impl ClusterSpec<'a, D>) -> Cluster<'a, C> {
        let mut next_node_id = self.next_node_id.borrow_mut();
        let id = *next_node_id;
        *next_node_id += 1;

        let cluster = spec.build(id);
        self.clusters.borrow_mut().push(cluster);

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

    pub fn spin<L: Location>(&self, on: &L) -> Stream<'a, (), Async, L> {
        Stream::new(
            on.location_kind(),
            self.ir_leaves().clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Spin(),
                location_kind: on.location_kind(),
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
            on.location_kind(),
            self.ir_leaves().clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Stream(e.into()),
                location_kind: on.location_kind(),
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
            on.location_kind(),
            self.ir_leaves().clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Iter(e.into()),
                location_kind: on.location_kind(),
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
            on.location_kind(),
            self.ir_leaves().clone(),
            HfPlusNode::Source {
                source: HfPlusSource::Interval(interval.into()),
                location_kind: on.location_kind(),
            },
        )
    }

    pub fn cycle<T, W, L: Location>(&self, on: &L) -> (HfCycle<'a, T, W, L>, Stream<'a, T, W, L>) {
        let next_id = {
            let on_id = match on.location_kind() {
                LocationKind::Process(id) => id,
                LocationKind::Cluster(id) => id,
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
                location_kind: on.location_kind(),
                ir_leaves: self.ir_leaves().clone(),
                _phantom: PhantomData,
            },
            Stream::new(
                on.location_kind(),
                self.ir_leaves().clone(),
                HfPlusNode::CycleSource {
                    ident,
                    location_kind: on.location_kind(),
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
    pub fn compile_no_network(mut self) -> HfCompiled<'a, D::GraphId> {
        self.used = true;

        HfCompiled {
            hydroflow_ir: build_inner(std::mem::take(&mut self.ir)),
            extra_stmts: BTreeMap::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_default_optimize(self) -> BuiltFlow<'a, D> {
        self.optimize_with(super::persist_pullup::persist_pullup)
    }
}

impl<'a, D: Deploy<'a>> BuiltFlow<'a, D> {
    pub fn compile(mut self, env: &D::CompileEnv) -> HfCompiled<'a, D::GraphId> {
        self.used = true;

        let location_to_node = self
            .nodes
            .iter()
            .map(|node| (node.id(), node))
            .collect::<HashMap<_, _>>();
        let location_to_cluster = self
            .clusters
            .iter()
            .map(|cluster| (cluster.id(), cluster))
            .collect::<HashMap<_, _>>();

        let mut seen_tees: HashMap<_, _> = HashMap::new();
        let ir_leaves_networked: Vec<HfPlusLeaf> = std::mem::take(&mut self.ir)
            .into_iter()
            .map(|leaf| {
                leaf.compile_network::<D>(
                    env,
                    &mut seen_tees,
                    &location_to_node,
                    &location_to_cluster,
                )
            })
            .collect();

        let all_locations_count = location_to_node.len() + location_to_cluster.len();

        let mut extra_stmts: BTreeMap<usize, Vec<syn::Stmt>> = BTreeMap::new();
        for cluster in &self.clusters {
            let self_id_ident = syn::Ident::new(
                &format!("__hydroflow_plus_cluster_self_id_{}", cluster.id()),
                Span::call_site(),
            );
            let self_id_expr = D::cluster_self_id(env).splice();
            extra_stmts
                .entry(cluster.id())
                .or_default()
                .push(syn::parse_quote! {
                    let #self_id_ident = #self_id_expr;
                });

            for other_location in 0..all_locations_count {
                let self_id = cluster.id();
                let other_id_ident = syn::Ident::new(
                    &format!("__hydroflow_plus_cluster_ids_{}", self_id),
                    Span::call_site(),
                );
                let other_id_expr = D::cluster_ids(env, self_id).splice();
                extra_stmts
                    .entry(other_location)
                    .or_default()
                    .push(syn::parse_quote! {
                        let #other_id_ident = #other_id_expr;
                    });
            }
        }

        HfCompiled {
            hydroflow_ir: build_inner(ir_leaves_networked),
            extra_stmts,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn deploy(mut self, env: &mut D::InstantiateEnv) -> DeployResult<'a, D>
    where
        D: Deploy<'a, CompileEnv = ()>,
    {
        self.used = true;

        let location_to_node = self
            .nodes
            .iter()
            .map(|node| (node.id(), node))
            .collect::<HashMap<_, _>>();
        let location_to_cluster = self
            .clusters
            .iter()
            .map(|cluster| (cluster.id(), cluster))
            .collect::<HashMap<_, _>>();

        let mut seen_tees_instantiate: HashMap<_, _> = HashMap::new();
        let ir_leaves_networked: Vec<HfPlusLeaf> = std::mem::take(&mut self.ir)
            .into_iter()
            .map(|leaf| {
                leaf.compile_network::<D>(
                    &(),
                    &mut seen_tees_instantiate,
                    &location_to_node,
                    &location_to_cluster,
                )
            })
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

        DeployResult {
            location_to_process: nodes.into_iter().map(|node| (node.id(), node)).collect(),
            location_to_cluster: clusters
                .into_iter()
                .map(|cluster| (cluster.id(), cluster))
                .collect(),
        }
    }
}

pub struct DeployResult<'a, D: Deploy<'a>> {
    pub(crate) location_to_process: HashMap<usize, D::Process>,
    pub(crate) location_to_cluster: HashMap<usize, D::Cluster>,
}

impl<'a, D: Deploy<'a>> DeployResult<'a, D> {
    pub fn get_process<P>(&self, p: Process<P>) -> &D::Process {
        let id = match p.location_kind() {
            LocationKind::Process(id) => id,
            LocationKind::Cluster(id) => id,
        };

        self.location_to_process.get(&id).unwrap()
    }

    pub fn get_cluster<C>(&self, c: Cluster<'a, C>) -> &D::Cluster {
        let id = match c.location_kind() {
            LocationKind::Process(id) => id,
            LocationKind::Cluster(id) => id,
        };

        self.location_to_cluster.get(&id).unwrap()
    }
}
