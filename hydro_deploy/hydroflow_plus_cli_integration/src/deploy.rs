use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use async_channel::Receiver;
use hydro_deploy::custom_service::CustomClientPort;
use hydro_deploy::hydroflow_crate::ports::{
    DemuxSink, HydroflowSink, HydroflowSource, TaggedSource,
};
use hydro_deploy::hydroflow_crate::HydroflowCrateService;
use hydro_deploy::{Deployment, Host};
use hydroflow_plus::builder::Builders;
use hydroflow_plus::node::{
    ClusterBuilder, Deploy, HfCluster, HfNode, HfSendManyToMany, HfSendManyToOne, HfSendOneToMany,
    HfSendOneToOne, NodeBuilder,
};
use hydroflow_plus::GraphBuilder;
use stageleft::internal::syn::parse_quote;
use stageleft::q;
use tokio::sync::RwLock;

use super::HydroflowPlusMeta;

pub struct CLIDeploy {}

impl<'a> Deploy<'a> for CLIDeploy {
    type Node = CLIDeployNode<'a>;
    type Cluster = CLIDeployCluster<'a>;
    type Meta = HashMap<usize, Vec<u32>>;
    type RuntimeID = ();
    type NodePort = CLIDeployPort<CLIDeployNode<'a>>;
    type ClusterPort = CLIDeployPort<CLIDeployCluster<'a>>;
}

pub trait DeployCrateWrapper {
    fn underlying(&self) -> Arc<RwLock<HydroflowCrateService>>;

    #[allow(async_fn_in_trait)]
    async fn create_sender(
        &self,
        port: &str,
        deployment: &mut Deployment,
        on: &Arc<RwLock<impl Host + 'static>>,
    ) -> CustomClientPort {
        let sender_service = deployment.CustomService(on.clone(), vec![]);
        let mut sender_port = sender_service.read().await.declare_client(&sender_service);
        let mut recipient = self
            .underlying()
            .read()
            .await
            .get_port(port.to_string(), &self.underlying());

        sender_port.send_to(&mut recipient);
        sender_port
    }

    #[allow(async_fn_in_trait)]
    async fn stdout(&self) -> Receiver<String> {
        self.underlying().read().await.stdout().await
    }

    #[allow(async_fn_in_trait)]
    async fn stderr(&self) -> Receiver<String> {
        self.underlying().read().await.stderr().await
    }
}

#[derive(Clone)]
pub struct CLIDeployNode<'a> {
    id: usize,
    builder: &'a GraphBuilder<'a, CLIDeploy>,
    next_port: Rc<RefCell<usize>>,
    underlying: Arc<RwLock<HydroflowCrateService>>,
}

impl<'a> DeployCrateWrapper for CLIDeployNode<'a> {
    fn underlying(&self) -> Arc<RwLock<HydroflowCrateService>> {
        self.underlying.clone()
    }
}

pub struct CLIDeployPort<N> {
    node: N,
    port: String,
}

impl<'a> CLIDeployPort<CLIDeployNode<'a>> {
    pub async fn create_sender(
        &self,
        deployment: &mut Deployment,
        on: &Arc<RwLock<impl Host + 'static>>,
    ) -> CustomClientPort {
        self.node.create_sender(&self.port, deployment, on).await
    }
}

impl<'a> HfNode<'a> for CLIDeployNode<'a> {
    type Port = CLIDeployPort<Self>;
    type Meta = HashMap<usize, Vec<u32>>;

    fn id(&self) -> usize {
        self.id
    }

    fn graph_builder(&self) -> (&'a RefCell<usize>, &'a Builders) {
        self.builder.builder_components()
    }

    fn next_port(&self) -> CLIDeployPort<Self> {
        let next_port = *self.next_port.borrow();
        *self.next_port.borrow_mut() += 1;

        CLIDeployPort {
            node: self.clone(),
            port: format!("port_{}", next_port),
        }
    }

    fn update_meta(&mut self, meta: &Self::Meta) {
        let mut n = self.underlying.try_write().unwrap();
        n.update_meta(
            serde_json::to_string(&HydroflowPlusMeta {
                clusters: meta.clone(),
                subgraph_id: self.id,
            })
            .unwrap(),
        );
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
pub struct CLIDeployCluster<'a> {
    id: usize,
    builder: &'a GraphBuilder<'a, CLIDeploy>,
    next_port: Rc<RefCell<usize>>,
    pub nodes: Vec<DeployClusterNode>,
}

impl<'a> HfNode<'a> for CLIDeployCluster<'a> {
    type Port = CLIDeployPort<Self>;
    type Meta = HashMap<usize, Vec<u32>>;

    fn id(&self) -> usize {
        self.id
    }

    fn graph_builder(&self) -> (&'a RefCell<usize>, &'a Builders) {
        self.builder.builder_components()
    }

    fn next_port(&self) -> CLIDeployPort<Self> {
        let next_port = *self.next_port.borrow();
        *self.next_port.borrow_mut() += 1;

        CLIDeployPort {
            node: self.clone(),
            port: format!("port_{}", next_port),
        }
    }

    fn update_meta(&mut self, meta: &Self::Meta) {
        let json_meta = serde_json::to_string(&HydroflowPlusMeta {
            clusters: meta.clone(),
            subgraph_id: self.id,
        })
        .unwrap();

        self.nodes.iter().for_each(|n| {
            let mut n = n.underlying.try_write().unwrap();
            n.update_meta(json_meta.clone());
        });
    }
}

impl<'a> HfCluster<'a> for CLIDeployCluster<'a> {
    fn ids(&self) -> impl stageleft::Quoted<'a, &'a Vec<u32>> + Copy + 'a {
        q!(panic!())
    }
}

impl<'a> HfSendOneToOne<'a, CLIDeployNode<'a>> for CLIDeployNode<'a> {
    fn connect(
        &self,
        other: &CLIDeployNode<'a>,
        source_port: &CLIDeployPort<CLIDeployNode<'a>>,
        recipient_port: &CLIDeployPort<CLIDeployNode<'a>>,
    ) {
        let mut source_port = self
            .underlying
            .try_read()
            .unwrap()
            .get_port(source_port.port.clone(), &self.underlying);

        let mut recipient_port = other
            .underlying
            .try_read()
            .unwrap()
            .get_port(recipient_port.port.clone(), &other.underlying);

        source_port.send_to(&mut recipient_port);
    }

    fn gen_sink_statement(&self, _port: &Self::Port) -> hydroflow_plus::lang::parse::Pipeline {
        parse_quote!(null())
    }

    fn gen_source_statement(
        _other: &CLIDeployNode<'a>,
        _port: &Self::Port,
    ) -> hydroflow_plus::lang::parse::Pipeline {
        parse_quote!(null())
    }
}

impl<'a> HfSendManyToOne<'a, CLIDeployNode<'a>> for CLIDeployCluster<'a> {
    fn connect(
        &self,
        other: &CLIDeployNode<'a>,
        source_port: &CLIDeployPort<CLIDeployCluster<'a>>,
        recipient_port: &CLIDeployPort<CLIDeployNode<'a>>,
    ) {
        let mut recipient_port = other
            .underlying
            .try_read()
            .unwrap()
            .get_port(recipient_port.port.clone(), &other.underlying)
            .merge();

        for (i, node) in self.nodes.iter().enumerate() {
            let source_port = node
                .underlying
                .try_read()
                .unwrap()
                .get_port(source_port.port.clone(), &node.underlying);

            TaggedSource {
                source: Arc::new(RwLock::new(source_port)),
                tag: i as u32,
            }
            .send_to(&mut recipient_port);
        }
    }

    fn gen_sink_statement(&self, _port: &Self::Port) -> hydroflow_plus::lang::parse::Pipeline {
        parse_quote!(null())
    }

    fn gen_source_statement(
        _other: &CLIDeployNode<'a>,
        _port: &CLIDeployPort<CLIDeployNode<'a>>,
    ) -> hydroflow_plus::lang::parse::Pipeline {
        parse_quote!(null())
    }
}

impl<'a> HfSendOneToMany<'a, CLIDeployCluster<'a>> for CLIDeployNode<'a> {
    fn connect(
        &self,
        other: &CLIDeployCluster<'a>,
        source_port: &CLIDeployPort<CLIDeployNode<'a>>,
        recipient_port: &CLIDeployPort<CLIDeployCluster<'a>>,
    ) {
        let mut source_port = self
            .underlying
            .try_read()
            .unwrap()
            .get_port(source_port.port.clone(), &self.underlying);

        let mut recipient_port = DemuxSink {
            demux: other
                .nodes
                .iter()
                .enumerate()
                .map(|(id, c)| {
                    let n = c.underlying.try_read().unwrap();
                    (
                        id as u32,
                        Arc::new(RwLock::new(
                            n.get_port(recipient_port.port.clone(), &c.underlying),
                        )) as Arc<RwLock<dyn HydroflowSink + 'static>>,
                    )
                })
                .collect(),
        };

        source_port.send_to(&mut recipient_port);
    }

    fn gen_sink_statement(&self, _port: &Self::Port) -> hydroflow_plus::lang::parse::Pipeline {
        parse_quote!(null())
    }

    fn gen_source_statement(
        _other: &CLIDeployCluster<'a>,
        _port: &CLIDeployPort<CLIDeployCluster<'a>>,
    ) -> hydroflow_plus::lang::parse::Pipeline {
        parse_quote!(null())
    }
}

impl<'a> HfSendManyToMany<'a, CLIDeployCluster<'a>> for CLIDeployCluster<'a> {
    fn connect(
        &self,
        other: &CLIDeployCluster<'a>,
        source_port: &CLIDeployPort<CLIDeployCluster<'a>>,
        recipient_port: &CLIDeployPort<CLIDeployCluster<'a>>,
    ) {
        for (i, sender) in self.nodes.iter().enumerate() {
            let source_port = sender
                .underlying
                .try_read()
                .unwrap()
                .get_port(source_port.port.clone(), &sender.underlying);

            let mut recipient_port = DemuxSink {
                demux: other
                    .nodes
                    .iter()
                    .enumerate()
                    .map(|(id, c)| {
                        let n = c.underlying.try_read().unwrap();
                        (
                            id as u32,
                            Arc::new(RwLock::new(
                                n.get_port(recipient_port.port.clone(), &c.underlying)
                                    .merge(),
                            ))
                                as Arc<RwLock<dyn HydroflowSink + 'static>>,
                        )
                    })
                    .collect(),
            };

            TaggedSource {
                source: Arc::new(RwLock::new(source_port)),
                tag: i as u32,
            }
            .send_to(&mut recipient_port);
        }
    }

    fn gen_sink_statement(&self, _port: &Self::Port) -> hydroflow_plus::lang::parse::Pipeline {
        parse_quote!(null())
    }

    fn gen_source_statement(
        _other: &CLIDeployCluster<'a>,
        _port: &CLIDeployPort<CLIDeployCluster<'a>>,
    ) -> hydroflow_plus::lang::parse::Pipeline {
        parse_quote!(null())
    }
}

type CrateBuilder<'a> = dyn FnMut() -> Arc<RwLock<HydroflowCrateService>> + 'a;

pub struct CLIDeployNodeBuilder<'a>(RefCell<Box<CrateBuilder<'a>>>);

impl<'a> CLIDeployNodeBuilder<'a> {
    pub fn new<F: FnMut() -> Arc<RwLock<HydroflowCrateService>> + 'a>(f: F) -> Self {
        Self(RefCell::new(Box::new(f)))
    }
}

impl<'a: 'b, 'b> NodeBuilder<'a, CLIDeploy> for CLIDeployNodeBuilder<'b> {
    fn build(
        &self,
        id: usize,
        builder: &'a GraphBuilder<'a, CLIDeploy>,
        _meta: &mut HashMap<usize, Vec<u32>>,
    ) -> CLIDeployNode<'a> {
        CLIDeployNode {
            id,
            builder,
            next_port: Rc::new(RefCell::new(0)),
            underlying: (self.0.borrow_mut())(),
        }
    }
}

type ClusterBuilderFn<'a> = dyn FnMut() -> Vec<Arc<RwLock<HydroflowCrateService>>> + 'a;

pub struct CLIDeployClusterBuilder<'a>(RefCell<Box<ClusterBuilderFn<'a>>>);

impl<'a> CLIDeployClusterBuilder<'a> {
    pub fn new<F: FnMut() -> Vec<Arc<RwLock<HydroflowCrateService>>> + 'a>(f: F) -> Self {
        Self(RefCell::new(Box::new(f)))
    }
}

impl<'a: 'b, 'b> ClusterBuilder<'a, CLIDeploy> for CLIDeployClusterBuilder<'b> {
    fn build(
        &self,
        id: usize,
        builder: &'a GraphBuilder<'a, CLIDeploy>,
        meta: &mut HashMap<usize, Vec<u32>>,
    ) -> CLIDeployCluster<'a> {
        let cluster_nodes = (self.0.borrow_mut())();
        meta.insert(id, (0..(cluster_nodes.len() as u32)).collect());

        CLIDeployCluster {
            id,
            builder,
            next_port: Rc::new(RefCell::new(0)),
            nodes: cluster_nodes
                .into_iter()
                .map(|u| DeployClusterNode { underlying: u })
                .collect(),
        }
    }
}
