use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use hydro_deploy::custom_service::CustomClientPort;
use hydro_deploy::hydroflow_crate::ports::{
    DemuxSink, HydroflowSink, HydroflowSource, TaggedSource,
};
use hydro_deploy::hydroflow_crate::HydroflowCrateService;
use hydro_deploy::{Deployment, Host};
use hydroflow_plus::lang::graph::HydroflowGraph;
use hydroflow_plus::location::{
    Cluster, ClusterSpec, Deploy, HfSendManyToMany, HfSendManyToOne, HfSendOneToMany,
    HfSendOneToOne, Location, ProcessSpec,
};
use stageleft::internal::syn::parse_quote;
use stageleft::q;
use tokio::sync::RwLock;

use super::HydroflowPlusMeta;

pub struct HydroDeploy {}

impl<'a> Deploy<'a> for HydroDeploy {
    type ClusterId = u32;
    type InstantiateEnv = Deployment;
    type Process = DeployNode;
    type Cluster = DeployCluster;
    type Meta = HashMap<usize, Vec<u32>>;
    type GraphId = ();
    type ProcessPort = DeployPort<DeployNode>;
    type ClusterPort = DeployPort<DeployCluster>;
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
pub struct DeployNode {
    id: usize,
    next_port: Rc<RefCell<usize>>,
    node_fn: Rc<CrateBuilder<'static>>,
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
        for member in self.node.members.borrow().iter() {
            out.push(member.create_sender(&self.port, deployment, on).await);
        }

        out
    }
}

impl Location for DeployNode {
    type Port = DeployPort<Self>;
    type Meta = HashMap<usize, Vec<u32>>;
    type InstantiateEnv = Deployment;

    fn id(&self) -> usize {
        self.id
    }

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
        _graph: HydroflowGraph,
    ) {
        *self.underlying.borrow_mut() = Some((self.node_fn.as_ref())(env));
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
    cluster_fn: Rc<
        RefCell<Box<dyn Fn(&mut Deployment) -> Vec<Arc<RwLock<HydroflowCrateService>>> + 'static>>,
    >,
    members: Rc<RefCell<Vec<DeployClusterNode>>>,
}

impl DeployCluster {
    pub fn members(&self) -> Vec<DeployClusterNode> {
        self.members.borrow().clone()
    }
}

impl Location for DeployCluster {
    type Port = DeployPort<Self>;
    type Meta = HashMap<usize, Vec<u32>>;
    type InstantiateEnv = Deployment;

    fn id(&self) -> usize {
        self.id
    }

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
        _env: &mut Self::InstantiateEnv,
        meta: &mut Self::Meta,
        _graph: HydroflowGraph,
    ) {
        let cluster_nodes = (self.cluster_fn.borrow_mut())(_env);
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

impl<'a> Cluster<'a> for DeployCluster {
    type Id = u32;

    fn ids(&self) -> impl stageleft::Quoted<'a, &'a Vec<u32>> + Copy + 'a {
        q!(panic!())
    }

    fn self_id(&self) -> impl stageleft::Quoted<'a, Self::Id> + Copy + 'a {
        q!(panic!())
    }
}

impl HfSendOneToOne<DeployNode> for DeployNode {
    fn connect(
        &self,
        other: &DeployNode,
        source_port: &DeployPort<DeployNode>,
        recipient_port: &DeployPort<DeployNode>,
    ) {
        let self_underlying_borrow = self.underlying.borrow();
        let self_underlying = self_underlying_borrow.as_ref().unwrap();
        let source_port = self_underlying
            .try_read()
            .unwrap()
            .get_port(source_port.port.clone(), self_underlying);

        let other_underlying_borrow = other.underlying.borrow();
        let other_underlying = other_underlying_borrow.as_ref().unwrap();
        let recipient_port = other_underlying
            .try_read()
            .unwrap()
            .get_port(recipient_port.port.clone(), &other_underlying);

        source_port.send_to(&recipient_port);
    }

    fn gen_sink_statement(&self, _port: &Self::Port) -> syn::Expr {
        parse_quote!(null)
    }

    fn gen_source_statement(_other: &DeployNode, _port: &Self::Port) -> syn::Expr {
        parse_quote!(null)
    }
}

impl HfSendManyToOne<DeployNode, u32> for DeployCluster {
    fn connect(
        &self,
        other: &DeployNode,
        source_port: &DeployPort<DeployCluster>,
        recipient_port: &DeployPort<DeployNode>,
    ) {
        let other_underlying_borrow = other.underlying.borrow();
        let other_underlying = other_underlying_borrow.as_ref().unwrap();
        let recipient_port = other_underlying
            .try_read()
            .unwrap()
            .get_port(recipient_port.port.clone(), &other_underlying)
            .merge();

        for (i, node) in self.members.borrow().iter().enumerate() {
            let source_port = node
                .underlying
                .try_read()
                .unwrap()
                .get_port(source_port.port.clone(), &node.underlying);

            TaggedSource {
                source: Arc::new(source_port),
                tag: i as u32,
            }
            .send_to(&recipient_port);
        }
    }

    fn gen_sink_statement(&self, _port: &Self::Port) -> syn::Expr {
        parse_quote!(null)
    }

    fn gen_source_statement(_other: &DeployNode, _port: &DeployPort<DeployNode>) -> syn::Expr {
        parse_quote!(null)
    }
}

impl HfSendOneToMany<DeployCluster, u32> for DeployNode {
    fn connect(
        &self,
        other: &DeployCluster,
        source_port: &DeployPort<DeployNode>,
        recipient_port: &DeployPort<DeployCluster>,
    ) {
        let self_underlying_borrow = self.underlying.borrow();
        let self_underlying = self_underlying_borrow.as_ref().unwrap();
        let source_port = self_underlying
            .try_read()
            .unwrap()
            .get_port(source_port.port.clone(), self_underlying);

        let recipient_port = DemuxSink {
            demux: other
                .members
                .borrow()
                .iter()
                .enumerate()
                .map(|(id, c)| {
                    let n = c.underlying.try_read().unwrap();
                    (
                        id as u32,
                        Arc::new(n.get_port(recipient_port.port.clone(), &c.underlying))
                            as Arc<dyn HydroflowSink + 'static>,
                    )
                })
                .collect(),
        };

        source_port.send_to(&recipient_port);
    }

    fn gen_sink_statement(&self, _port: &Self::Port) -> syn::Expr {
        parse_quote!(null)
    }

    fn gen_source_statement(
        _other: &DeployCluster,
        _port: &DeployPort<DeployCluster>,
    ) -> syn::Expr {
        parse_quote!(null)
    }
}

impl HfSendManyToMany<DeployCluster, u32> for DeployCluster {
    fn connect(
        &self,
        other: &DeployCluster,
        source_port: &DeployPort<DeployCluster>,
        recipient_port: &DeployPort<DeployCluster>,
    ) {
        for (i, sender) in self.members.borrow().iter().enumerate() {
            let source_port = sender
                .underlying
                .try_read()
                .unwrap()
                .get_port(source_port.port.clone(), &sender.underlying);

            let recipient_port = DemuxSink {
                demux: other
                    .members
                    .borrow()
                    .iter()
                    .enumerate()
                    .map(|(id, c)| {
                        let n = c.underlying.try_read().unwrap();
                        (
                            id as u32,
                            Arc::new(
                                n.get_port(recipient_port.port.clone(), &c.underlying)
                                    .merge(),
                            ) as Arc<dyn HydroflowSink + 'static>,
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

    fn gen_sink_statement(&self, _port: &Self::Port) -> syn::Expr {
        parse_quote!(null)
    }

    fn gen_source_statement(
        _other: &DeployCluster,
        _port: &DeployPort<DeployCluster>,
    ) -> syn::Expr {
        parse_quote!(null)
    }
}

type CrateBuilder<'a> = dyn Fn(&mut Deployment) -> Arc<RwLock<HydroflowCrateService>> + 'a;

pub struct DeployProcessSpec(Rc<CrateBuilder<'static>>);

impl DeployProcessSpec {
    pub fn new<F: Fn(&mut Deployment) -> Arc<RwLock<HydroflowCrateService>> + 'static>(
        f: F,
    ) -> Self {
        Self(Rc::new(f))
    }
}

impl<'a> ProcessSpec<'a, HydroDeploy> for DeployProcessSpec {
    fn build(&self, id: usize) -> DeployNode {
        DeployNode {
            id,
            next_port: Rc::new(RefCell::new(0)),
            node_fn: self.0.clone(),
            underlying: Rc::new(RefCell::new(None)),
        }
    }
}

type ClusterSpecFn = dyn Fn(&mut Deployment) -> Vec<Arc<RwLock<HydroflowCrateService>>> + 'static;

pub struct DeployClusterSpec(Rc<RefCell<Box<ClusterSpecFn>>>);

impl DeployClusterSpec {
    pub fn new<F: Fn(&mut Deployment) -> Vec<Arc<RwLock<HydroflowCrateService>>> + 'static>(
        f: F,
    ) -> Self {
        Self(Rc::new(RefCell::new(Box::new(f))))
    }
}

impl<'a> ClusterSpec<'a, HydroDeploy> for DeployClusterSpec {
    fn build(&self, id: usize) -> DeployCluster {
        DeployCluster {
            id,
            next_port: Rc::new(RefCell::new(0)),
            cluster_fn: self.0.clone(),
            members: Rc::new(RefCell::new(vec![])),
        }
    }
}
