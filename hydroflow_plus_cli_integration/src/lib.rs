use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use hydro_cli::core::hydroflow_crate::ports::HydroflowSource;
use hydro_cli::core::HydroflowCrate;
use hydroflow_plus::node::{HFDeploy, HFNodeBuilder, HfConnectable, HfNode};
use hydroflow_plus::HfBuilder;
use stageleft::RuntimeData;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CLIDeployNode {
    id: usize,
    next_port: Rc<RefCell<usize>>,
    pub underlying: Arc<RwLock<HydroflowCrate>>,
}

impl CLIDeployNode {
    pub fn new(id: usize, underlying: Arc<RwLock<HydroflowCrate>>) -> Self {
        Self {
            id,
            next_port: Rc::new(RefCell::new(0)),
            underlying,
        }
    }
}

impl<'a> HfNode<'a> for CLIDeployNode {
    fn id(&self) -> usize {
        self.id
    }

    fn next_port(&self) -> String {
        let next_port = *self.next_port.borrow();
        *self.next_port.borrow_mut() += 1;
        format!("port_{}", next_port)
    }

    fn get_cli(&self) -> RuntimeData<&'a hydroflow_plus::util::cli::HydroCLI> {
        Default::default()
    }
}

impl<'a> HfConnectable<'a, CLIDeployNode> for CLIDeployNode {
    fn connect(&self, other: &CLIDeployNode, source_port: &str, recipient_port: &str) {
        let mut source_port = self
            .underlying
            .try_read()
            .unwrap()
            .get_port(source_port.to_string(), &self.underlying);

        let mut recipient_port = other
            .underlying
            .try_read()
            .unwrap()
            .get_port(recipient_port.to_string(), &other.underlying);

        source_port.send_to(&mut recipient_port);
    }
}

pub struct CLIDeployNodeBuilder<'a>(Box<dyn FnMut(usize) -> Arc<RwLock<HydroflowCrate>> + 'a>);

impl<'a> CLIDeployNodeBuilder<'a> {
    pub fn new<F: FnMut(usize) -> Arc<RwLock<HydroflowCrate>> + 'a>(f: F) -> Self {
        Self(Box::new(f))
    }
}

impl<'a, 'b> HFNodeBuilder<'a, CLIDeployNode> for CLIDeployNodeBuilder<'b> {
    fn build(&mut self, builder: &'a HfBuilder<'a>) -> CLIDeployNode {
        let id = builder.next_node_id();
        CLIDeployNode::new(id, (self.0)(id))
    }
}

pub struct CLIDeploy<'b> {
    _marker: std::marker::PhantomData<&'b ()>,
}

impl<'a, 'b> HFDeploy<'a> for CLIDeploy<'b> {
    type Node = CLIDeployNode;
    type NodeBuilder = CLIDeployNodeBuilder<'b>;
}
