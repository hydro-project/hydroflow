use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;

use proc_macro2::Span;
use stageleft::Quoted;

use super::built::build_inner;
use crate::deploy::{LocalDeploy, Node};
use crate::ir::HfPlusLeaf;
use crate::location::{Location, LocationId};
use crate::{Cluster, ClusterSpec, Deploy, HfCompiled, Process, ProcessSpec};

pub struct DeployFlow<'a, D: LocalDeploy<'a>> {
    pub(super) ir: Vec<HfPlusLeaf<'a>>,
    pub(super) nodes: HashMap<usize, D::Process>,
    pub(super) clusters: HashMap<usize, D::Cluster>,
    pub(super) used: bool,

    pub(super) _phantom: PhantomData<&'a mut &'a D>,
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
        for &c_id in self.clusters.keys() {
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
}

impl<'a, D: Deploy<'a, CompileEnv = ()>> DeployFlow<'a, D> {
    #[must_use]
    pub fn deploy(mut self, env: &mut D::InstantiateEnv) -> DeployResult<'a, D> {
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

        let (mut processes, mut clusters) = (
            std::mem::take(&mut self.nodes)
                .into_iter()
                .map(|(node_id, node)| {
                    node.instantiate(env, &mut meta, compiled.remove(&node_id).unwrap());
                    (node_id, node)
                })
                .collect::<HashMap<_, _>>(),
            std::mem::take(&mut self.clusters)
                .into_iter()
                .map(|(cluster_id, cluster)| {
                    cluster.instantiate(env, &mut meta, compiled.remove(&cluster_id).unwrap());
                    (cluster_id, cluster)
                })
                .collect::<HashMap<_, _>>(),
        );

        for node in processes.values_mut() {
            node.update_meta(&meta);
        }

        for cluster in clusters.values_mut() {
            cluster.update_meta(&meta);
        }

        let mut seen_tees_connect = HashMap::new();
        for leaf in ir_leaves_networked {
            leaf.connect_network(&mut seen_tees_connect);
        }

        DeployResult {
            processes,
            clusters,
        }
    }
}

pub struct DeployResult<'a, D: Deploy<'a>> {
    processes: HashMap<usize, D::Process>,
    clusters: HashMap<usize, D::Cluster>,
}

impl<'a, D: Deploy<'a>> DeployResult<'a, D> {
    pub fn get_process<P>(&self, p: Process<P>) -> &D::Process {
        let id = match p.id() {
            LocationId::Process(id) => id,
            LocationId::Cluster(id) => id,
        };

        self.processes.get(&id).unwrap()
    }

    pub fn get_cluster<C>(&self, c: Cluster<'a, C>) -> &D::Cluster {
        let id = match c.id() {
            LocationId::Process(id) => id,
            LocationId::Cluster(id) => id,
        };

        self.clusters.get(&id).unwrap()
    }
}
