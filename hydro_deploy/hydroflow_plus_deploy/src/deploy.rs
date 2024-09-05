use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use hydro_deploy::custom_service::CustomClientPort;
use hydro_deploy::hydroflow_crate::ports::{
    DemuxSink, HydroflowSink, HydroflowSource, TaggedSource,
};
use hydro_deploy::hydroflow_crate::tracing_options::TracingOptions;
use hydro_deploy::hydroflow_crate::HydroflowCrateService;
use hydro_deploy::{Deployment, Host, HydroflowCrate};
use hydroflow_plus::deploy::{ClusterSpec, Deploy, Node, ProcessSpec};
use hydroflow_plus::lang::graph::HydroflowGraph;
use nameof::name_of;
use sha2::{Digest, Sha256};
use stageleft::{Quoted, RuntimeData};
use tokio::sync::RwLock;

use super::HydroflowPlusMeta;
use crate::deploy_runtime::*;
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
        let p1_port = p1_port.port.as_str();
        let p2_port = p2_port.port.as_str();
        deploy_o2o(
            RuntimeData::new("__hydroflow_plus_trybuild_cli"),
            p1_port,
            p2_port,
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
        let p1_port = p1_port.port.as_str();
        let c2_port = c2_port.port.as_str();
        deploy_o2m(
            RuntimeData::new("__hydroflow_plus_trybuild_cli"),
            p1_port,
            c2_port,
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
        let c1_port = c1_port.port.as_str();
        let p2_port = p2_port.port.as_str();
        deploy_m2o(
            RuntimeData::new("__hydroflow_plus_trybuild_cli"),
            c1_port,
            p2_port,
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
        let c1_port = c1_port.port.as_str();
        let c2_port = c2_port.port.as_str();
        deploy_m2m(
            RuntimeData::new("__hydroflow_plus_trybuild_cli"),
            c1_port,
            c2_port,
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
        cluster_members(
            RuntimeData::new("__hydroflow_plus_trybuild_cli"),
            of_cluster,
        )
    }

    fn cluster_self_id(_env: &Self::CompileEnv) -> impl Quoted<'a, u32> + Copy + 'a {
        cluster_self_id(RuntimeData::new("__hydroflow_plus_trybuild_cli"))
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
    pub rustflags: Option<String>,
    pub tracing: Option<TracingOptions>,
    pub name_hint: Option<String>,
    pub cluster_idx: Option<usize>,
}

impl TrybuildHost {
    pub fn new(host: Arc<dyn Host>) -> Self {
        Self {
            host,
            display_name: None,
            rustflags: None,
            tracing: None,
            name_hint: None,
            cluster_idx: None,
        }
    }

    pub fn display_name(self, display_name: impl Into<String>) -> Self {
        if self.display_name.is_some() {
            panic!("{} already set", name_of!(display_name in Self));
        }

        Self {
            display_name: Some(display_name.into()),
            ..self
        }
    }

    pub fn rustflags(self, rustflags: impl Into<String>) -> Self {
        if self.rustflags.is_some() {
            panic!("{} already set", name_of!(rustflags in Self));
        }

        Self {
            rustflags: Some(rustflags.into()),
            ..self
        }
    }

    pub fn tracing(self, tracing: TracingOptions) -> Self {
        if self.tracing.is_some() {
            panic!("{} already set", name_of!(tracing in Self));
        }

        Self {
            tracing: Some(tracing),
            ..self
        }
    }
}

impl From<Arc<dyn Host>> for TrybuildHost {
    fn from(h: Arc<dyn Host>) -> Self {
        Self {
            host: h,
            display_name: None,
            rustflags: None,
            tracing: None,
            name_hint: None,
            cluster_idx: None,
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
                let (bin_name, (dir, target_dir, features)) =
                    create_graph_trybuild(graph, extra_stmts, &trybuild.name_hint);
                create_trybuild_service(trybuild, &dir, &target_dir, &features, &bin_name)
            }
        };

        *self.underlying.borrow_mut() = Some(env.add_service(service));
    }
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
    name_hint: Option<String>,
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
            Some(create_graph_trybuild(graph, extra_stmts, &self.name_hint))
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
                        let (bin_name, (dir, target_dir, features)) =
                            maybe_trybuild.as_ref().unwrap();
                        create_trybuild_service(trybuild, dir, target_dir, features, bin_name)
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
    fn build(self, id: usize, _name_hint: &str) -> DeployNode {
        DeployNode {
            id,
            next_port: Rc::new(RefCell::new(0)),
            service_spec: Rc::new(RefCell::new(Some(CrateOrTrybuild::Crate(self.0)))),
            underlying: Rc::new(RefCell::new(None)),
        }
    }
}

impl<'a> ProcessSpec<'a, HydroDeploy> for TrybuildHost {
    fn build(mut self, id: usize, name_hint: &str) -> DeployNode {
        self.name_hint = Some(format!("{} (process {id})", name_hint));
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
    fn build(self, id: usize, _name_hint: &str) -> DeployCluster {
        DeployCluster {
            id,
            next_port: Rc::new(RefCell::new(0)),
            cluster_spec: Rc::new(RefCell::new(Some(
                self.0.into_iter().map(CrateOrTrybuild::Crate).collect(),
            ))),
            members: Rc::new(RefCell::new(vec![])),
            name_hint: None,
        }
    }
}

impl<'a> ClusterSpec<'a, HydroDeploy> for Vec<TrybuildHost> {
    fn build(self, id: usize, name_hint: &str) -> DeployCluster {
        let name_hint = format!("{} (cluster {id})", name_hint);
        DeployCluster {
            id,
            next_port: Rc::new(RefCell::new(0)),
            cluster_spec: Rc::new(RefCell::new(Some(
                self.into_iter()
                    .enumerate()
                    .map(|(idx, mut b)| {
                        b.name_hint = Some(name_hint.clone());
                        b.cluster_idx = Some(idx);
                        CrateOrTrybuild::Trybuild(b)
                    })
                    .collect(),
            ))),
            members: Rc::new(RefCell::new(vec![])),
            name_hint: Some(name_hint),
        }
    }
}

fn clean_name_hint(name_hint: &str) -> String {
    name_hint
        .replace("::", "__")
        .replace(" ", "_")
        .replace(",", "_")
        .replace("<", "_")
        .replace(">", "")
        .replace("(", "")
        .replace(")", "")
}

fn create_graph_trybuild(
    graph: HydroflowGraph,
    extra_stmts: Vec<syn::Stmt>,
    name_hint: &Option<String>,
) -> (
    String,
    (std::path::PathBuf, std::path::PathBuf, Option<Vec<String>>),
) {
    let source_ast = compile_graph_trybuild(graph, extra_stmts);

    let source_dir = trybuild_internals_api::cargo::manifest_dir().unwrap();
    let source_manifest = trybuild_internals_api::dependencies::get_manifest(&source_dir).unwrap();
    let crate_name = &source_manifest.package.name.to_string().replace("-", "_");
    let source = prettyplease::unparse(&source_ast)
        .to_string()
        .replace(crate_name, &format!("{crate_name}::__staged"))
        .replace("crate::__staged", &format!("{crate_name}::__staged"));

    let mut hasher = Sha256::new();
    hasher.update(&source);
    let hash = format!("{:X}", hasher.finalize())
        .chars()
        .take(8)
        .collect::<String>();

    let bin_name = if let Some(name_hint) = &name_hint {
        format!("{}_{}", clean_name_hint(name_hint), &hash)
    } else {
        hash
    };

    let trybuild_created = create_trybuild(&source, &bin_name).unwrap();
    (bin_name, trybuild_created)
}

fn create_trybuild_service(
    trybuild: TrybuildHost,
    dir: &std::path::PathBuf,
    target_dir: &std::path::PathBuf,
    features: &Option<Vec<String>>,
    bin_name: &str,
) -> HydroflowCrate {
    let mut ret = HydroflowCrate::new(dir, trybuild.host)
        .target_dir(target_dir)
        .bin(bin_name)
        .no_default_features();

    if let Some(display_name) = trybuild.display_name {
        ret = ret.display_name(display_name);
    } else if let Some(name_hint) = trybuild.name_hint {
        if let Some(cluster_idx) = trybuild.cluster_idx {
            ret = ret.display_name(format!("{} / {}", name_hint, cluster_idx));
        } else {
            ret = ret.display_name(name_hint);
        }
    }

    if let Some(rustflags) = trybuild.rustflags {
        ret = ret.rustflags(rustflags);
    }

    if let Some(tracing) = trybuild.tracing {
        ret = ret.tracing(tracing);
    }

    if let Some(features) = features {
        ret = ret.features(features.clone());
    }

    ret
}
