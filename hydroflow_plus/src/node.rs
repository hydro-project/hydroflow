use std::cell::RefCell;
use std::rc::Rc;

use hydroflow::util::cli::HydroCLI;
use stageleft::RuntimeData;

pub trait HfNode<'a>: Clone {
    fn id(&self) -> usize;
    fn next_port(&self) -> String;
    fn get_cli(&self) -> RuntimeData<&'a HydroCLI>;
}

impl<'a> HfNode<'a> for () {
    fn id(&self) -> usize {
        0
    }

    fn next_port(&self) -> String {
        panic!();
    }

    fn get_cli(&self) -> RuntimeData<&'a HydroCLI> {
        panic!();
    }
}

impl<'a> HfNode<'a> for usize {
    fn id(&self) -> usize {
        *self
    }

    fn next_port(&self) -> String {
        panic!();
    }

    fn get_cli(&self) -> RuntimeData<&'a HydroCLI> {
        panic!();
    }
}

#[derive(Clone)]
pub struct CLIRuntimeNode<'a> {
    id: usize,
    next_port: Rc<RefCell<usize>>,
    cli: RuntimeData<&'a HydroCLI>,
}

impl<'a> CLIRuntimeNode<'a> {
    pub fn new(id: usize, cli: RuntimeData<&'a HydroCLI>) -> CLIRuntimeNode {
        CLIRuntimeNode {
            id,
            next_port: Rc::new(RefCell::new(0)),
            cli,
        }
    }
}

impl<'a> HfNode<'a> for CLIRuntimeNode<'a> {
    fn id(&self) -> usize {
        self.id
    }

    fn next_port(&self) -> String {
        let next_send_port = *self.next_port.borrow();
        *self.next_port.borrow_mut() += 1;
        format!("port_{}", next_send_port)
    }

    fn get_cli(&self) -> RuntimeData<&'a HydroCLI> {
        self.cli
    }
}

pub trait HfConnectable<'a, O: HfNode<'a>> {
    fn connect(&self, other: &O, source_port: &str, recipient_port: &str);
}

impl<'a> HfConnectable<'a, CLIRuntimeNode<'a>> for CLIRuntimeNode<'a> {
    fn connect(&self, _other: &CLIRuntimeNode, _source_port: &str, _recipient_port: &str) {}
}

pub trait HfNodeBuilder<N> {
    fn build(&mut self, id: usize) -> N;
}

pub struct CLIRuntimeNodeBuilder<'a> {
    cli: RuntimeData<&'a HydroCLI>,
}

impl CLIRuntimeNodeBuilder<'_> {
    pub fn new(cli: RuntimeData<&HydroCLI>) -> CLIRuntimeNodeBuilder {
        CLIRuntimeNodeBuilder { cli }
    }
}

impl<'a> HfNodeBuilder<CLIRuntimeNode<'a>> for CLIRuntimeNodeBuilder<'a> {
    fn build(&mut self, id: usize) -> CLIRuntimeNode<'a> {
        CLIRuntimeNode::new(id, self.cli)
    }
}

pub trait HfDeploy<'a> {
    type Node: HfNode<'a>;
    type NodeBuilder: HfNodeBuilder<Self::Node>;
}

pub trait HfNetworkedDeploy<'a>: HfDeploy<'a, Node = Self::NetworkedNode> {
    type NetworkedNode: HfNode<'a> + HfConnectable<'a, Self::NetworkedNode>;
}

impl<'a, T: HfDeploy<'a, Node = N>, N: HfNode<'a> + HfConnectable<'a, N>> HfNetworkedDeploy<'a>
    for T
{
    type NetworkedNode = N;
}

pub struct SingleGraph<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> HfDeploy<'a> for SingleGraph<'a> {
    type Node = ();
    type NodeBuilder = ();
}

impl HfNodeBuilder<()> for () {
    fn build(&mut self, _id: usize) {}
}

pub struct MultiGraph<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> HfDeploy<'a> for MultiGraph<'a> {
    type Node = usize;
    type NodeBuilder = ();
}

impl HfNodeBuilder<usize> for () {
    fn build(&mut self, id: usize) -> usize {
        id
    }
}

pub struct CLIRuntime<'b> {
    _marker: std::marker::PhantomData<&'b ()>,
}

impl<'a: 'b, 'b> HfDeploy<'a> for CLIRuntime<'b> {
    type Node = CLIRuntimeNode<'a>;
    type NodeBuilder = CLIRuntimeNodeBuilder<'a>;
}
