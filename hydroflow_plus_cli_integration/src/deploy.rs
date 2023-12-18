use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use async_channel::Receiver;
use hydro_cli::core::custom_service::CustomClientPort;
use hydro_cli::core::hydroflow_crate::ports::HydroflowSource;
use hydro_cli::core::{Deployment, Host, HydroflowCrate};
use hydroflow_plus::builder::Builders;
use hydroflow_plus::node::{
    HfCluster, HfClusterBuilder, HfDemuxTo, HfDeploy, HfNode, HfNodeBuilder, HfSendTo,
};
use hydroflow_plus::HfBuilder;
use stageleft::internal::syn::parse_quote;
use stageleft::q;
use tokio::sync::RwLock;

use crate::HydroflowPlusMeta;

pub struct CLIDeploy {}

impl<'a> HfDeploy<'a> for CLIDeploy {
    type Node = CLIDeployNode<'a>;
    type Cluster = CLIDeployCluster<'a>;
    type Meta = HydroflowPlusMeta;
    type RuntimeID = ();
}

#[derive(Clone)]
pub struct CLIDeployNode<'a> {
    id: usize,
    builder: &'a HfBuilder<'a, CLIDeploy>,
    next_port: Rc<RefCell<usize>>,
    underlying: Arc<RwLock<HydroflowCrate>>,
}

impl<'a> CLIDeployNode<'a> {
    pub async fn create_sender(
        &self,
        port: &str,
        deployment: &mut Deployment,
        on: &Arc<RwLock<impl Host + 'static>>,
    ) -> CustomClientPort {
        let sender_service = deployment.CustomService(on.clone(), vec![]);
        let mut sender_port = sender_service.read().await.declare_client(&sender_service);
        let mut recipient = self
            .underlying
            .read()
            .await
            .get_port(port.to_string(), &self.underlying);

        sender_port.send_to(&mut recipient);
        sender_port
    }

    pub async fn stdout(&self) -> Receiver<String> {
        self.underlying.read().await.stdout().await
    }

    pub async fn stderr(&self) -> Receiver<String> {
        self.underlying.read().await.stderr().await
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
pub struct CLIDeployCluster<'a> {
    id: usize,
    builder: &'a HfBuilder<'a, CLIDeploy>,
    next_port: Rc<RefCell<usize>>,
    _underlying: Vec<Arc<RwLock<HydroflowCrate>>>,
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
            self._underlying.iter().for_each(|n| {
                let mut n = n.try_write().unwrap();
                n.set_meta(serde_json::to_string(&meta).unwrap());
            });
        }
    }
}

impl<'a> HfCluster<'a> for CLIDeployCluster<'a> {
    fn ids(&self) -> impl stageleft::Quoted<'a, &'a Vec<usize>> {
        q!(panic!())
    }
}

impl<'a> HfSendTo<'a, CLIDeployNode<'a>> for CLIDeployNode<'a> {
    fn send_to(
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

impl<'a> HfDemuxTo<'a, CLIDeployCluster<'a>> for CLIDeployNode<'a> {
    fn demux_to(
        &self,
        other: &CLIDeployCluster<'a>,
        source_port: &CLIDeployPort<CLIDeployNode<'a>>,
        recipient_port: &CLIDeployPort<CLIDeployCluster<'a>>,
    ) {
        todo!()
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

pub struct CLIDeployNodeBuilder<'a>(Box<dyn FnMut(usize) -> Arc<RwLock<HydroflowCrate>> + 'a>);

impl<'a> CLIDeployNodeBuilder<'a> {
    pub fn new<F: FnMut(usize) -> Arc<RwLock<HydroflowCrate>> + 'a>(f: F) -> Self {
        Self(Box::new(f))
    }
}

impl<'a: 'b, 'b> HfNodeBuilder<'a, CLIDeploy> for CLIDeployNodeBuilder<'b> {
    fn build(&mut self, id: usize, builder: &'a HfBuilder<'a, CLIDeploy>) -> CLIDeployNode<'a> {
        CLIDeployNode {
            id,
            builder,
            next_port: Rc::new(RefCell::new(0)),
            underlying: (self.0)(id),
        }
    }
}

pub struct CLIDeployClusterBuilder<'a>(
    Box<dyn FnMut(usize) -> Vec<Arc<RwLock<HydroflowCrate>>> + 'a>,
);

impl<'a> CLIDeployClusterBuilder<'a> {
    pub fn new<F: FnMut(usize) -> Vec<Arc<RwLock<HydroflowCrate>>> + 'a>(f: F) -> Self {
        Self(Box::new(f))
    }
}

impl<'a: 'b, 'b> HfClusterBuilder<'a, CLIDeploy> for CLIDeployClusterBuilder<'b> {
    fn build(&mut self, id: usize, builder: &'a HfBuilder<'a, CLIDeploy>) -> CLIDeployCluster<'a> {
        let cluster_nodes = (self.0)(id);
        builder
            .meta
            .borrow_mut()
            .get_or_insert(HydroflowPlusMeta {
                clusters: HashMap::new(),
            })
            .clusters
            .insert(id, (0..cluster_nodes.len()).collect());

        CLIDeployCluster {
            id,
            builder,
            next_port: Rc::new(RefCell::new(0)),
            _underlying: cluster_nodes,
        }
    }
}
