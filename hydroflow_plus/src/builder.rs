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

    pub fn finalize(mut self) -> BuiltFlow<'a> {
        self.finalized = true;

        BuiltFlow {
            ir: self.ir_leaves.borrow_mut().take().unwrap(),
            nodes: self.nodes.replace(vec![]),
            clusters: self.clusters.replace(vec![]),
            used: false,
            _phantom: PhantomData,
        }
    }

    pub fn with_default_optimize(self) -> BuiltFlow<'a> {
        self.finalize().with_default_optimize()
    }

    pub fn optimize_with(
        self,
        f: impl FnOnce(Vec<HfPlusLeaf<'a>>) -> Vec<HfPlusLeaf<'a>>,
    ) -> BuiltFlow<'a> {
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

    pub fn cluster<C>(&self) -> Cluster<'a, C> {
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

pub struct BuiltFlow<'a> {
    pub(crate) ir: Vec<HfPlusLeaf<'a>>,
    nodes: Vec<usize>,
    clusters: Vec<usize>,
    used: bool,

    _phantom: PhantomData<&'a mut &'a ()>,
}

impl<'a> Drop for BuiltFlow<'a> {
    fn drop(&mut self) {
        if !self.used {
            panic!("Dropped BuiltFlow without instantiating, you may have forgotten to call `compile` or `deploy`.");
        }
    }
}

impl<'a> BuiltFlow<'a> {
    pub fn ir(&self) -> &Vec<HfPlusLeaf<'a>> {
        &self.ir
    }

    pub fn optimize_with(
        mut self,
        f: impl FnOnce(Vec<HfPlusLeaf<'a>>) -> Vec<HfPlusLeaf<'a>>,
    ) -> BuiltFlow<'a> {
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

impl<'a> BuiltFlow<'a> {
    pub fn compile_no_network<D: LocalDeploy<'a>>(mut self) -> HfCompiled<'a, D::GraphId> {
        self.used = true;

        HfCompiled {
            hydroflow_ir: build_inner(std::mem::take(&mut self.ir)),
            extra_stmts: BTreeMap::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_default_optimize(self) -> BuiltFlow<'a> {
        self.optimize_with(super::persist_pullup::persist_pullup)
    }

    fn into_deploy<D: LocalDeploy<'a>>(mut self) -> DeployFlow<'a, D> {
        self.used = true;
        let nodes = if D::has_default_node() {
            self.nodes
                .iter()
                .map(|id| (*id, D::default_process(*id)))
                .collect()
        } else {
            HashMap::new()
        };

        let clusters = if D::has_default_node() {
            self.clusters
                .iter()
                .map(|id| (*id, D::default_cluster(*id)))
                .collect()
        } else {
            HashMap::new()
        };

        DeployFlow {
            ir: std::mem::take(&mut self.ir),
            nodes,
            clusters,
            used: false,
            _phantom: PhantomData,
        }
    }

    pub fn compile<D: Deploy<'a> + 'a>(self, env: &D::CompileEnv) -> HfCompiled<'a, D::GraphId> {
        self.into_deploy::<D>().compile(env)
    }

    pub fn with_process<P, D: LocalDeploy<'a>>(
        self,
        process: &Process<P>,
        spec: impl ProcessSpec<'a, D>,
    ) -> DeployFlow<'a, D> {
        self.into_deploy().with_process(process, spec)
    }

    pub fn with_cluster<C, D: LocalDeploy<'a>>(
        self,
        cluster: &Cluster<'a, C>,
        spec: impl ClusterSpec<'a, D>,
    ) -> DeployFlow<'a, D> {
        self.into_deploy().with_cluster(cluster, spec)
    }
}

pub struct DeployFlow<'a, D: LocalDeploy<'a>> {
    pub(crate) ir: Vec<HfPlusLeaf<'a>>,
    nodes: HashMap<usize, D::Process>,
    clusters: HashMap<usize, D::Cluster>,
    used: bool,

    _phantom: PhantomData<&'a mut &'a D>,
}

impl<'a, D: LocalDeploy<'a>> Drop for DeployFlow<'a, D> {
    fn drop(&mut self) {
        if !self.used {
            panic!("Dropped DeployFlow without instantiating, you may have forgotten to call `compile` or `deploy`.");
        }
    }
}

impl<'a, D: LocalDeploy<'a>> DeployFlow<'a, D> {
    pub fn with_process<P>(mut self, process: &Process<P>, spec: impl ProcessSpec<'a, D>) -> Self {
        self.nodes.insert(process.id, spec.build(process.id));
        self
    }

    pub fn with_cluster<C>(
        mut self,
        cluster: &Cluster<'a, C>,
        spec: impl ClusterSpec<'a, D>,
    ) -> Self {
        self.clusters.insert(cluster.id, spec.build(cluster.id));
        self
    }
}

impl<'a, D: Deploy<'a>> DeployFlow<'a, D> {
    pub fn compile(mut self, env: &D::CompileEnv) -> HfCompiled<'a, D::GraphId> {
        self.used = true;

        let mut seen_tees: HashMap<_, _> = HashMap::new();
        let ir_leaves_networked: Vec<HfPlusLeaf> = std::mem::take(&mut self.ir)
            .into_iter()
            .map(|leaf| leaf.compile_network::<D>(env, &mut seen_tees, &self.nodes, &self.clusters))
            .collect();

        let all_locations_count = self.nodes.len() + self.clusters.len();

        let mut extra_stmts: BTreeMap<usize, Vec<syn::Stmt>> = BTreeMap::new();
        for (&c_id, _) in &self.clusters {
            let self_id_ident = syn::Ident::new(
                &format!("__hydroflow_plus_cluster_self_id_{}", c_id),
                Span::call_site(),
            );
            let self_id_expr = D::cluster_self_id(env).splice();
            extra_stmts
                .entry(c_id)
                .or_default()
                .push(syn::parse_quote! {
                    let #self_id_ident = #self_id_expr;
                });

            for other_location in 0..all_locations_count {
                let other_id_ident = syn::Ident::new(
                    &format!("__hydroflow_plus_cluster_ids_{}", c_id),
                    Span::call_site(),
                );
                let other_id_expr = D::cluster_ids(env, c_id).splice();
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

        let mut seen_tees_instantiate: HashMap<_, _> = HashMap::new();
        let ir_leaves_networked: Vec<HfPlusLeaf> = std::mem::take(&mut self.ir)
            .into_iter()
            .map(|leaf| {
                leaf.compile_network::<D>(
                    &(),
                    &mut seen_tees_instantiate,
                    &self.nodes,
                    &self.clusters,
                )
            })
            .collect();

        let mut compiled = build_inner(ir_leaves_networked.clone());
        let mut meta = D::Meta::default();

        let (mut nodes, mut clusters): (Vec<(usize, D::Process)>, Vec<(usize, D::Cluster)>) = (
            std::mem::take(&mut self.nodes)
                .into_iter()
                .map(|(node_id, node)| {
                    node.instantiate(env, &mut meta, compiled.remove(&node_id).unwrap());
                    (node_id, node)
                })
                .collect(),
            std::mem::take(&mut self.clusters)
                .into_iter()
                .map(|(cluster_id, cluster)| {
                    cluster.instantiate(env, &mut meta, compiled.remove(&cluster_id).unwrap());
                    (cluster_id, cluster)
                })
                .collect(),
        );

        for (_, node) in &mut nodes {
            node.update_meta(&meta);
        }

        for (_, cluster) in &mut clusters {
            cluster.update_meta(&meta);
        }

        let mut seen_tees_connect = HashMap::new();
        for leaf in ir_leaves_networked {
            leaf.connect_network(&mut seen_tees_connect);
        }

        DeployResult {
            location_to_process: nodes.into_iter().map(|(id, node)| (id, node)).collect(),
            location_to_cluster: clusters
                .into_iter()
                .map(|(id, cluster)| (id, cluster))
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
