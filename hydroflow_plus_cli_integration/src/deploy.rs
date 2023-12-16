use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use async_channel::Receiver;
use hydro_cli::core::custom_service::CustomClientPort;
use hydro_cli::core::hydroflow_crate::ports::HydroflowSource;
use hydro_cli::core::{Deployment, Host, HydroflowCrate};
use hydroflow_plus::builder::Builders;
use hydroflow_plus::node::{HfDeploy, HfNode, HfNodeBuilder, HfSendTo};
use hydroflow_plus::HfBuilder;
use stageleft::internal::syn::parse_quote;
use tokio::sync::RwLock;

pub struct CLIDeploy {}

impl<'a> HfDeploy<'a> for CLIDeploy {
    type Node = CLIDeployNode<'a>;
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

pub struct CLIDeployPort<'a> {
    node: CLIDeployNode<'a>,
    port: String,
}

impl<'a> CLIDeployPort<'a> {
    pub async fn create_sender(
        &self,
        deployment: &mut Deployment,
        on: &Arc<RwLock<impl Host + 'static>>,
    ) -> CustomClientPort {
        self.node.create_sender(&self.port, deployment, on).await
    }
}

impl<'a> HfNode<'a> for CLIDeployNode<'a> {
    type Port = CLIDeployPort<'a>;

    fn id(&self) -> usize {
        self.id
    }

    fn graph_builder(&self) -> (&'a RefCell<usize>, &'a Builders) {
        self.builder.builder_components()
    }

    fn next_port(&self) -> CLIDeployPort<'a> {
        let next_port = *self.next_port.borrow();
        *self.next_port.borrow_mut() += 1;

        CLIDeployPort {
            node: self.clone(),
            port: format!("port_{}", next_port),
        }
    }

    fn gen_sink_statement(&self, _port: &Self::Port) -> hydroflow_plus::lang::parse::Pipeline {
        parse_quote!(null())
    }

    fn gen_source_statement(&self, _port: &Self::Port) -> hydroflow_plus::lang::parse::Pipeline {
        parse_quote!(null())
    }
}

impl<'a> HfSendTo<'a, CLIDeployNode<'a>> for CLIDeployNode<'a> {
    fn send_to(
        &self,
        other: &CLIDeployNode<'a>,
        source_port: &CLIDeployPort<'a>,
        recipient_port: &CLIDeployPort<'a>,
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
