use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;

use hydroflow_lang::graph::{eliminate_extra_unions_tees, HydroflowGraph};
use proc_macro2::Span;
use stageleft::Quoted;

use crate::deploy::{ClusterSpec, Deploy, LocalDeploy, Node, ProcessSpec};
use crate::ir::HfPlusLeaf;
use crate::location::{Cluster, Location, LocationId, Process};
use crate::HfCompiled;

pub struct BuiltFlow<'a> {
    pub(crate) ir: Vec<HfPlusLeaf<'a>>,
    pub(crate) nodes: Vec<usize>,
    pub(crate) clusters: Vec<usize>,
    pub(crate) used: bool,

    pub(crate) _phantom: PhantomData<&'a mut &'a ()>,
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

pub(crate) fn build_inner(ir: Vec<HfPlusLeaf>) -> BTreeMap<usize, HydroflowGraph> {
    let mut builders = BTreeMap::new();
    let mut built_tees = HashMap::new();
    let mut next_stmt_id = 0;
    for leaf in ir {
        leaf.emit(&mut builders, &mut built_tees, &mut next_stmt_id);
    }

    builders
        .into_iter()
        .map(|(k, v)| {
            let (mut flat_graph, _, _) = v.build();
            eliminate_extra_unions_tees(&mut flat_graph);
            (k, flat_graph)
        })
        .collect()
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
        self.optimize_with(crate::persist_pullup::persist_pullup)
    }

    pub(crate) fn into_deploy<D: LocalDeploy<'a>>(mut self) -> DeployFlow<'a, D> {
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
    pub(crate) nodes: HashMap<usize, D::Process>,
    pub(crate) clusters: HashMap<usize, D::Cluster>,
    pub(crate) used: bool,

    pub(crate) _phantom: PhantomData<&'a mut &'a D>,
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
        let id = match p.id() {
            LocationId::Process(id) => id,
            LocationId::Cluster(id) => id,
        };

        self.location_to_process.get(&id).unwrap()
    }

    pub fn get_cluster<C>(&self, c: Cluster<'a, C>) -> &D::Cluster {
        let id = match c.id() {
            LocationId::Process(id) => id,
            LocationId::Cluster(id) => id,
        };

        self.location_to_cluster.get(&id).unwrap()
    }
}
