use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use async_channel::Receiver;
use hydro_cli::core::custom_service::CustomClientPort;
use hydro_cli::core::hydroflow_crate::ports::HydroflowSource;
use hydro_cli::core::{Deployment, Host, HydroflowCrate};
use hydroflow_plus::node::{HfDeploy, HfNode, HfNodeBuilder, HfSendTo};
use stageleft::internal::syn::parse_quote;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CLIDeployNode {
    id: usize,
    next_port: Rc<RefCell<usize>>,
    underlying: Arc<RwLock<HydroflowCrate>>,
}

impl CLIDeployNode {
    pub fn new(id: usize, underlying: Arc<RwLock<HydroflowCrate>>) -> Self {
        Self {
            id,
            next_port: Rc::new(RefCell::new(0)),
            underlying,
        }
    }

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

pub struct CLIDeployPort {
    node: CLIDeployNode,
    port: String,
}

impl CLIDeployPort {
    pub async fn create_sender(
        &self,
        deployment: &mut Deployment,
        on: &Arc<RwLock<impl Host + 'static>>,
    ) -> CustomClientPort {
        self.node.create_sender(&self.port, deployment, on).await
    }
}

impl<'a> HfNode<'a> for CLIDeployNode {
    type Port = CLIDeployPort;

    fn id(&self) -> usize {
        self.id
    }

    fn next_port(&self) -> CLIDeployPort {
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

impl<'a> HfSendTo<'a, CLIDeployNode> for CLIDeployNode {
    fn send_to(
        &self,
        other: &CLIDeployNode,
        source_port: &CLIDeployPort,
        recipient_port: &CLIDeployPort,
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

impl<'a> HfNodeBuilder<CLIDeployNode> for CLIDeployNodeBuilder<'a> {
    fn build(&mut self, id: usize) -> CLIDeployNode {
        CLIDeployNode::new(id, (self.0)(id))
    }
}

pub struct CLIDeploy<'b> {
    _marker: std::marker::PhantomData<&'b ()>,
}

impl<'a, 'b> HfDeploy<'a> for CLIDeploy<'b> {
    type Node = CLIDeployNode;
    type NodeBuilder = CLIDeployNodeBuilder<'b>;
}
