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
    HfCluster, HfClusterBuilder, HfDeploy, HfNode, HfNodeBuilder, HfSendManyToMany,
    HfSendManyToOne, HfSendOneToMany, HfSendOneToOne,
};
use hydroflow_plus::HfBuilder;
use stageleft::internal::syn::parse_quote;
use stageleft::q;
use tokio::sync::RwLock;

use super::HydroflowPlusMeta;

pub struct CLIDeploy {}

impl<'a> HfDeploy<'a> for CLIDeploy {
    type Node = CLIDeployNode<'a>;
    type Cluster = CLIDeployCluster<'a>;
    type Meta = HydroflowPlusMeta;
    type RuntimeID = ();
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
    builder: &'a HfBuilder<'a, CLIDeploy>,
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
    type Meta = HydroflowPlusMeta;

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

    fn build(&mut self, meta: &Option<Self::Meta>) {
        if let Some(meta) = meta {
            let mut n = self.underlying.try_write().unwrap();
            n.set_meta(serde_json::to_string(&meta).unwrap());
        }
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
    builder: &'a HfBuilder<'a, CLIDeploy>,
    next_port: Rc<RefCell<usize>>,
    pub nodes: Vec<DeployClusterNode>,
}

impl<'a> HfNode<'a> for CLIDeployCluster<'a> {
    type Port = CLIDeployPort<Self>;
    type Meta = HydroflowPlusMeta;

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

    fn build(&mut self, meta: &Option<Self::Meta>) {
        if let Some(meta) = meta {
            self.nodes.iter().for_each(|n| {
                let mut n = n.underlying.try_write().unwrap();
                n.set_meta(serde_json::to_string(&meta).unwrap());
            });
        }
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

type CrateBuilder<'a> = dyn FnMut(usize) -> Arc<RwLock<HydroflowCrateService>> + 'a;

pub struct CLIDeployNodeBuilder<'a>(RefCell<Box<CrateBuilder<'a>>>);

impl<'a> CLIDeployNodeBuilder<'a> {
    pub fn new<F: FnMut(usize) -> Arc<RwLock<HydroflowCrateService>> + 'a>(f: F) -> Self {
        Self(RefCell::new(Box::new(f)))
    }
}

impl<'a: 'b, 'b> HfNodeBuilder<'a, CLIDeploy> for CLIDeployNodeBuilder<'b> {
    fn build(&self, id: usize, builder: &'a HfBuilder<'a, CLIDeploy>) -> CLIDeployNode<'a> {
        CLIDeployNode {
            id,
            builder,
            next_port: Rc::new(RefCell::new(0)),
            underlying: (self.0.borrow_mut())(id),
        }
    }
}

type ClusterBuilder<'a> = dyn FnMut(usize) -> Vec<Arc<RwLock<HydroflowCrateService>>> + 'a;

pub struct CLIDeployClusterBuilder<'a>(RefCell<Box<ClusterBuilder<'a>>>);

impl<'a> CLIDeployClusterBuilder<'a> {
    pub fn new<F: FnMut(usize) -> Vec<Arc<RwLock<HydroflowCrateService>>> + 'a>(f: F) -> Self {
        Self(RefCell::new(Box::new(f)))
    }
}

impl<'a: 'b, 'b> HfClusterBuilder<'a, CLIDeploy> for CLIDeployClusterBuilder<'b> {
    fn build(&self, id: usize, builder: &'a HfBuilder<'a, CLIDeploy>) -> CLIDeployCluster<'a> {
        let cluster_nodes = (self.0.borrow_mut())(id);
        builder
            .meta
            .borrow_mut()
            .get_or_insert(HydroflowPlusMeta {
                clusters: HashMap::new(),
            })
            .clusters
            .insert(id, (0..(cluster_nodes.len() as u32)).collect());

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
