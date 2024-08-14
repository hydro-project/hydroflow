use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use hydro_deploy::custom_service::CustomClientPort;
use hydro_deploy::hydroflow_crate::perf_options::PerfOptions;
use hydro_deploy::hydroflow_crate::ports::{
    DemuxSink, HydroflowSink, HydroflowSource, TaggedSource,
};
use hydro_deploy::hydroflow_crate::HydroflowCrateService;
use hydro_deploy::{Deployment, Host, HydroflowCrate};
use hydroflow_plus::deploy::{ClusterSpec, Deploy, Node, ProcessSpec};
use hydroflow_plus::lang::graph::HydroflowGraph;
use hydroflow_plus::util::cli::{
    ConnectedDemux, ConnectedDirect, ConnectedSink, ConnectedSource, ConnectedTagged, HydroCLI,
};
use sha2::{Digest, Sha256};
use stageleft::{q, Quoted, RuntimeData};
use tokio::sync::RwLock;

use super::HydroflowPlusMeta;
use crate::trybuild::{compile_graph_trybuild, create_trybuild};

pub struct HydroDeploy {}

impl<'a> Deploy<'a> for HydroDeploy {
    type InstantiateEnv = Deployment;
    type CompileEnv = ();
    type Process = DeployNode;
    type Cluster = DeployCluster;
    type Meta = HashMap<usize, Vec<u32>>;
    type GraphId = ();
    type ProcessPort = DeployPort<DeployNode>;
    type ClusterPort = DeployPort<DeployCluster>;

    fn allocate_process_port(process: &Self::Process) -> Self::ProcessPort {
        process.next_port()
    }

    fn allocate_cluster_port(cluster: &Self::Cluster) -> Self::ClusterPort {
        cluster.next_port()
    }

    fn o2o_sink_source(
        _env: &(),
        _p1: &Self::Process,
        p1_port: &Self::ProcessPort,
        _p2: &Self::Process,
        p2_port: &Self::ProcessPort,
    ) -> (syn::Expr, syn::Expr) {
        let env: RuntimeData<&HydroCLI<HydroflowPlusMeta>> =
            RuntimeData::new("__hydroflow_plus_trybuild_cli");
        (
            {
                let port = p1_port.port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedDirect>()
                        .into_sink()
                })
                .splice()
            },
            {
                let port = p2_port.port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedDirect>()
                        .into_source()
                })
                .splice()
            },
        )
    }

    fn o2o_connect(
        p1: &Self::Process,
        p1_port: &Self::ProcessPort,
        p2: &Self::Process,
        p2_port: &Self::ProcessPort,
    ) {
        let self_underlying_borrow = p1.underlying.borrow();
        let self_underlying = self_underlying_borrow.as_ref().unwrap();
        let source_port = self_underlying
            .try_read()
            .unwrap()
            .get_port(p1_port.port.clone(), self_underlying);

        let other_underlying_borrow = p2.underlying.borrow();
        let other_underlying = other_underlying_borrow.as_ref().unwrap();
        let recipient_port = other_underlying
            .try_read()
            .unwrap()
            .get_port(p2_port.port.clone(), other_underlying);

        source_port.send_to(&recipient_port);
    }

    fn o2m_sink_source(
        _env: &(),
        _p1: &Self::Process,
        p1_port: &Self::ProcessPort,
        _c2: &Self::Cluster,
        c2_port: &Self::ClusterPort,
    ) -> (syn::Expr, syn::Expr) {
        let env: RuntimeData<&HydroCLI<HydroflowPlusMeta>> =
            RuntimeData::new("__hydroflow_plus_trybuild_cli");
        (
            {
                let port = p1_port.port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedDemux<ConnectedDirect>>()
                        .into_sink()
                })
                .splice()
            },
            {
                let port = c2_port.port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedDirect>()
                        .into_source()
                })
                .splice()
            },
        )
    }

    fn o2m_connect(
        p1: &Self::Process,
        p1_port: &Self::ProcessPort,
        c2: &Self::Cluster,
        c2_port: &Self::ClusterPort,
    ) {
        let self_underlying_borrow = p1.underlying.borrow();
        let self_underlying = self_underlying_borrow.as_ref().unwrap();
        let source_port = self_underlying
            .try_read()
            .unwrap()
            .get_port(p1_port.port.clone(), self_underlying);

        let recipient_port = DemuxSink {
            demux: c2
                .members
                .borrow()
                .iter()
                .enumerate()
                .map(|(id, c)| {
                    let n = c.underlying.try_read().unwrap();
                    (
                        id as u32,
                        Arc::new(n.get_port(c2_port.port.clone(), &c.underlying))
                            as Arc<dyn HydroflowSink + 'static>,
                    )
                })
                .collect(),
        };

        source_port.send_to(&recipient_port);
    }

    fn m2o_sink_source(
        _env: &(),
        _c1: &Self::Cluster,
        c1_port: &Self::ClusterPort,
        _p2: &Self::Process,
        p2_port: &Self::ProcessPort,
    ) -> (syn::Expr, syn::Expr) {
        let env: RuntimeData<&HydroCLI<HydroflowPlusMeta>> =
            RuntimeData::new("__hydroflow_plus_trybuild_cli");
        (
            {
                let port = c1_port.port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedDirect>()
                        .into_sink()
                })
                .splice()
            },
            {
                let port = p2_port.port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedTagged<ConnectedDirect>>()
                        .into_source()
                })
                .splice()
            },
        )
    }

    fn m2o_connect(
        c1: &Self::Cluster,
        c1_port: &Self::ClusterPort,
        p2: &Self::Process,
        p2_port: &Self::ProcessPort,
    ) {
        let other_underlying_borrow = p2.underlying.borrow();
        let other_underlying = other_underlying_borrow.as_ref().unwrap();
        let recipient_port = other_underlying
            .try_read()
            .unwrap()
            .get_port(p2_port.port.clone(), other_underlying)
            .merge();

        for (i, node) in c1.members.borrow().iter().enumerate() {
            let source_port = node
                .underlying
                .try_read()
                .unwrap()
                .get_port(c1_port.port.clone(), &node.underlying);

            TaggedSource {
                source: Arc::new(source_port),
                tag: i as u32,
            }
            .send_to(&recipient_port);
        }
    }

    fn m2m_sink_source(
        _env: &(),
        _c1: &Self::Cluster,
        c1_port: &Self::ClusterPort,
        _c2: &Self::Cluster,
        c2_port: &Self::ClusterPort,
    ) -> (syn::Expr, syn::Expr) {
        let env: RuntimeData<&HydroCLI<HydroflowPlusMeta>> =
            RuntimeData::new("__hydroflow_plus_trybuild_cli");
        (
            {
                let port = c1_port.port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedDemux<ConnectedDirect>>()
                        .into_sink()
                })
                .splice()
            },
            {
                let port = c2_port.port.as_str();

                q!({
                    env.port(port)
                        .connect_local_blocking::<ConnectedTagged<ConnectedDirect>>()
                        .into_source()
                })
                .splice()
            },
        )
    }

    fn m2m_connect(
        c1: &Self::Cluster,
        c1_port: &Self::ClusterPort,
        c2: &Self::Cluster,
        c2_port: &Self::ClusterPort,
    ) {
        for (i, sender) in c1.members.borrow().iter().enumerate() {
            let source_port = sender
                .underlying
                .try_read()
                .unwrap()
                .get_port(c1_port.port.clone(), &sender.underlying);

            let recipient_port = DemuxSink {
                demux: c2
                    .members
                    .borrow()
                    .iter()
                    .enumerate()
                    .map(|(id, c)| {
                        let n = c.underlying.try_read().unwrap();
                        (
                            id as u32,
                            Arc::new(n.get_port(c2_port.port.clone(), &c.underlying).merge())
                                as Arc<dyn HydroflowSink + 'static>,
                        )
                    })
                    .collect(),
            };

            TaggedSource {
                source: Arc::new(source_port),
                tag: i as u32,
            }
            .send_to(&recipient_port);
        }
    }

    fn cluster_ids(
        _env: &Self::CompileEnv,
        of_cluster: usize,
    ) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a {
        let cli: RuntimeData<&HydroCLI<HydroflowPlusMeta>> =
            RuntimeData::new("__hydroflow_plus_trybuild_cli");
        q!(cli.meta.clusters.get(&of_cluster).unwrap())
    }

    fn cluster_self_id(_env: &Self::CompileEnv) -> impl Quoted<'a, u32> + Copy + 'a {
        let cli: RuntimeData<&HydroCLI<HydroflowPlusMeta>> =
            RuntimeData::new("__hydroflow_plus_trybuild_cli");
        q!(cli
            .meta
            .cluster_id
            .expect("Tried to read Cluster ID on a non-cluster node"))
    }
}

pub trait DeployCrateWrapper {
    fn underlying(&self) -> Arc<RwLock<HydroflowCrateService>>;

    #[allow(async_fn_in_trait)]
    async fn create_sender(
        &self,
        port: &str,
        deployment: &mut Deployment,
        on: &Arc<impl Host + 'static>,
    ) -> CustomClientPort {
        let sender_service = deployment.CustomService(on.clone(), vec![]);
        let sender_port = sender_service.read().await.declare_client(&sender_service);
        let recipient = self
            .underlying()
            .read()
            .await
            .get_port(port.to_string(), &self.underlying());

        sender_port.send_to(&recipient);
        sender_port
    }

    #[allow(async_fn_in_trait)]
    async fn stdout(&self) -> tokio::sync::mpsc::UnboundedReceiver<String> {
        self.underlying().read().await.stdout()
    }

    #[allow(async_fn_in_trait)]
    async fn stderr(&self) -> tokio::sync::mpsc::UnboundedReceiver<String> {
        self.underlying().read().await.stderr()
    }
}

#[derive(Clone)]
pub struct TrybuildHost {
    pub host: Arc<dyn Host>,
    pub display_name: Option<String>,
    pub profile: Option<String>,
    pub perf: Option<PerfOptions>,
}

impl TrybuildHost {
    pub fn new(host: Arc<dyn Host>) -> Self {
        Self {
            host,
            display_name: None,
            profile: None,
            perf: None,
        }
    }

    pub fn display_name(self, name: impl Into<String>) -> Self {
        if self.display_name.is_some() {
            panic!("display_name already set");
        }

        Self {
            display_name: Some(name.into()),
            ..self
        }
    }

    pub fn profile(self, profile: impl Into<String>) -> Self {
        if self.profile.is_some() {
            panic!("profile already set");
        }

        Self {
            profile: Some(profile.into()),
            ..self
        }
    }

    pub fn perf(self, perf: PerfOptions) -> Self {
        if self.perf.is_some() {
            panic!("perf already set");
        }

        Self {
            perf: Some(perf),
            ..self
        }
    }
}

impl From<Arc<dyn Host>> for TrybuildHost {
    fn from(h: Arc<dyn Host>) -> Self {
        Self {
            host: h,
            display_name: None,
            profile: None,
            perf: None,
        }
    }
}

pub enum CrateOrTrybuild {
    Crate(HydroflowCrate),
    Trybuild(TrybuildHost),
}

#[derive(Clone)]
pub struct DeployNode {
    id: usize,
    next_port: Rc<RefCell<usize>>,
    service_spec: Rc<RefCell<Option<CrateOrTrybuild>>>,
    underlying: Rc<RefCell<Option<Arc<RwLock<HydroflowCrateService>>>>>,
}

impl DeployCrateWrapper for DeployNode {
    fn underlying(&self) -> Arc<RwLock<HydroflowCrateService>> {
        self.underlying.borrow().as_ref().unwrap().clone()
    }
}

pub struct DeployPort<N> {
    node: N,
    port: String,
}

impl DeployPort<DeployNode> {
    pub async fn create_sender(
        &self,
        deployment: &mut Deployment,
        on: &Arc<impl Host + 'static>,
    ) -> CustomClientPort {
        self.node.create_sender(&self.port, deployment, on).await
    }
}

impl DeployPort<DeployCluster> {
    pub async fn create_senders(
        &self,
        deployment: &mut Deployment,
        on: &Arc<impl Host + 'static>,
    ) -> Vec<CustomClientPort> {
        let mut out = vec![];
        for member in self.node.members() {
            out.push(member.create_sender(&self.port, deployment, on).await);
        }

        out
    }
}

impl Node for DeployNode {
    type Port = DeployPort<Self>;
    type Meta = HashMap<usize, Vec<u32>>;
    type InstantiateEnv = Deployment;

    fn next_port(&self) -> DeployPort<Self> {
        let next_port = *self.next_port.borrow();
        *self.next_port.borrow_mut() += 1;

        DeployPort {
            node: self.clone(),
            port: format!("port_{}", next_port),
        }
    }

    fn update_meta(&mut self, meta: &Self::Meta) {
        let underlying_node = self.underlying.borrow();
        let mut n = underlying_node.as_ref().unwrap().try_write().unwrap();
        n.update_meta(HydroflowPlusMeta {
            clusters: meta.clone(),
            cluster_id: None,
            subgraph_id: self.id,
        });
    }

    fn instantiate(
        &self,
        env: &mut Self::InstantiateEnv,
        _meta: &mut Self::Meta,
        graph: HydroflowGraph,
        extra_stmts: Vec<syn::Stmt>,
    ) {
        let service = match self.service_spec.borrow_mut().take().unwrap() {
            CrateOrTrybuild::Crate(c) => c,
            CrateOrTrybuild::Trybuild(trybuild) => {
                let source_ast = compile_graph_trybuild(graph, extra_stmts);

                let source_dir = trybuild::cargo::manifest_dir().unwrap();
                let source_manifest = trybuild::dependencies::get_manifest(&source_dir).unwrap();
                let crate_name = &source_manifest.package.name.to_string().replace("-", "_");
                let source = prettyplease::unparse(&source_ast)
                    .to_string()
                    .replace(crate_name, &format!("{crate_name}::__staged"))
                    .replace("crate::__staged", &format!("{crate_name}::__staged"));

                let mut hasher = Sha256::new();
                hasher.update(&source);
                let hash = format!("{:X}", hasher.finalize());

                let (dir, target_dir, _features) = create_trybuild(&source, &hash).unwrap();
                create_trybuild_service(trybuild, &dir, &target_dir, &hash)
            }
        };

        *self.underlying.borrow_mut() = Some(env.add_service(service));
    }
}

fn create_trybuild_service(
    trybuild: TrybuildHost,
    dir: &std::path::PathBuf,
    target_dir: &std::path::PathBuf,
    hash: &str,
) -> HydroflowCrate {
    // TODO(shadaj): --no-default-features, --features
    let mut ret = HydroflowCrate::new(dir, trybuild.host)
        .target_dir(target_dir)
        .bin(hash);
    if let Some(display_name) = trybuild.display_name {
        ret = ret.display_name(display_name);
    }

    if let Some(profile) = trybuild.profile {
        ret = ret.profile(profile);
    }

    if let Some(perf) = trybuild.perf {
        ret = ret.perf(perf);
    }

    ret
}

#[derive(Clone)]
pub struct DeployClusterNode {
    underlying: Arc<RwLock<HydroflowCrateService>>,
}

impl DeployCrateWrapper for DeployClusterNode {
    fn underlying(&self) -> Arc<RwLock<HydroflowCrateService>> {
        self.underlying.clone()
    }
}

#[derive(Clone)]
pub struct DeployCluster {
    id: usize,
    next_port: Rc<RefCell<usize>>,
    cluster_spec: Rc<RefCell<Option<Vec<CrateOrTrybuild>>>>,
    members: Rc<RefCell<Vec<DeployClusterNode>>>,
}

impl DeployCluster {
    pub fn members(&self) -> Vec<DeployClusterNode> {
        self.members.borrow().clone()
    }
}

impl Node for DeployCluster {
    type Port = DeployPort<Self>;
    type Meta = HashMap<usize, Vec<u32>>;
    type InstantiateEnv = Deployment;

    fn next_port(&self) -> DeployPort<Self> {
        let next_port = *self.next_port.borrow();
        *self.next_port.borrow_mut() += 1;

        DeployPort {
            node: self.clone(),
            port: format!("port_{}", next_port),
        }
    }

    fn instantiate(
        &self,
        env: &mut Self::InstantiateEnv,
        meta: &mut Self::Meta,
        graph: HydroflowGraph,
        extra_stmts: Vec<syn::Stmt>,
    ) {
        let has_trybuild = self
            .cluster_spec
            .borrow()
            .as_ref()
            .unwrap()
            .iter()
            .any(|spec| matches!(spec, CrateOrTrybuild::Trybuild { .. }));

        let maybe_trybuild = if has_trybuild {
            let source_ast = compile_graph_trybuild(graph, extra_stmts);

            let source_dir = trybuild::cargo::manifest_dir().unwrap();
            let source_manifest = trybuild::dependencies::get_manifest(&source_dir).unwrap();
            let crate_name = &source_manifest.package.name;
            let source = prettyplease::unparse(&source_ast)
                .to_string()
                .replace("crate::__staged", &format!("{crate_name}::__staged"));

            let mut hasher = Sha256::new();
            hasher.update(&source);
            let hash = format!("{:X}", hasher.finalize());

            Some((hash.clone(), create_trybuild(&source, &hash).unwrap()))
        } else {
            None
        };

        let cluster_nodes = self
            .cluster_spec
            .borrow_mut()
            .take()
            .unwrap()
            .into_iter()
            .map(|spec| {
                let service = match spec {
                    CrateOrTrybuild::Crate(c) => c,
                    CrateOrTrybuild::Trybuild(trybuild) => {
                        let (hash, (dir, target_dir, _features)) = maybe_trybuild.as_ref().unwrap();
                        // TODO(shadaj): --no-default-features, --features
                        create_trybuild_service(trybuild, dir, target_dir, hash)
                    }
                };

                env.add_service(service)
            })
            .collect::<Vec<_>>();
        meta.insert(self.id, (0..(cluster_nodes.len() as u32)).collect());
        *self.members.borrow_mut() = cluster_nodes
            .into_iter()
            .map(|n| DeployClusterNode { underlying: n })
            .collect();
    }

    fn update_meta(&mut self, meta: &Self::Meta) {
        for (cluster_id, node) in self.members.borrow().iter().enumerate() {
            let mut n = node.underlying.try_write().unwrap();
            n.update_meta(HydroflowPlusMeta {
                clusters: meta.clone(),
                cluster_id: Some(cluster_id as u32),
                subgraph_id: self.id,
            });
        }
    }
}

#[derive(Clone)]
pub struct DeployProcessSpec(HydroflowCrate);

impl DeployProcessSpec {
    pub fn new(t: HydroflowCrate) -> Self {
        Self(t)
    }
}

impl<'a> ProcessSpec<'a, HydroDeploy> for DeployProcessSpec {
    fn build(self, id: usize) -> DeployNode {
        DeployNode {
            id,
            next_port: Rc::new(RefCell::new(0)),
            service_spec: Rc::new(RefCell::new(Some(CrateOrTrybuild::Crate(self.0)))),
            underlying: Rc::new(RefCell::new(None)),
        }
    }
}

impl<'a> ProcessSpec<'a, HydroDeploy> for TrybuildHost {
    fn build(self, id: usize) -> DeployNode {
        DeployNode {
            id,
            next_port: Rc::new(RefCell::new(0)),
            service_spec: Rc::new(RefCell::new(Some(CrateOrTrybuild::Trybuild(self)))),
            underlying: Rc::new(RefCell::new(None)),
        }
    }
}

#[derive(Clone)]
pub struct DeployClusterSpec(Vec<HydroflowCrate>);

impl DeployClusterSpec {
    pub fn new(crates: Vec<HydroflowCrate>) -> Self {
        Self(crates)
    }
}

impl<'a> ClusterSpec<'a, HydroDeploy> for DeployClusterSpec {
    fn build(self, id: usize) -> DeployCluster {
        DeployCluster {
            id,
            next_port: Rc::new(RefCell::new(0)),
            cluster_spec: Rc::new(RefCell::new(Some(
                self.0
                    .into_iter()
                    .map(|c| CrateOrTrybuild::Crate(c))
                    .collect(),
            ))),
            members: Rc::new(RefCell::new(vec![])),
        }
    }
}

impl<'a> ClusterSpec<'a, HydroDeploy> for Vec<TrybuildHost> {
    fn build(self, id: usize) -> DeployCluster {
        DeployCluster {
            id,
            next_port: Rc::new(RefCell::new(0)),
            cluster_spec: Rc::new(RefCell::new(Some(
                self.into_iter()
                    .map(|h| CrateOrTrybuild::Trybuild(h))
                    .collect(),
            ))),
            members: Rc::new(RefCell::new(vec![])),
        }
    }
}
