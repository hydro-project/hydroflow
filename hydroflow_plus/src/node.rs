use std::cell::RefCell;
use std::rc::Rc;

use hydroflow::util::cli::HydroCLI;
use hydroflow_lang::parse::Pipeline;
use proc_macro2::Span;
use quote::quote;
use stageleft::{Quoted, RuntimeData};
use syn::parse_quote;

pub trait HfNode<'a>: Clone {
    type Port;

    fn id(&self) -> usize;
    fn next_port(&self) -> Self::Port;
    fn gen_source_statement(&self, port: &Self::Port) -> Pipeline;
    fn gen_sink_statement(&self, port: &Self::Port) -> Pipeline;
}

impl<'a> HfNode<'a> for () {
    type Port = ();

    fn id(&self) -> usize {
        0
    }

    fn next_port(&self) {
        panic!();
    }

    fn gen_source_statement(&self, _port: &()) -> Pipeline {
        panic!();
    }

    fn gen_sink_statement(&self, _port: &()) -> Pipeline {
        panic!();
    }
}

impl<'a> HfNode<'a> for usize {
    type Port = ();

    fn id(&self) -> usize {
        *self
    }

    fn next_port(&self) {
        panic!();
    }

    fn gen_source_statement(&self, _port: &()) -> Pipeline {
        panic!();
    }

    fn gen_sink_statement(&self, _port: &()) -> Pipeline {
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
    type Port = String;

    fn id(&self) -> usize {
        self.id
    }

    fn next_port(&self) -> String {
        let next_send_port = *self.next_port.borrow();
        *self.next_port.borrow_mut() += 1;
        format!("port_{}", next_send_port)
    }

    fn gen_source_statement(&self, port: &String) -> Pipeline {
        let hydroflow_crate = proc_macro_crate::crate_name("hydroflow_plus")
            .expect("hydroflow_plus should be present in `Cargo.toml`");
        let root = match hydroflow_crate {
            proc_macro_crate::FoundCrate::Itself => quote! { hydroflow_plus },
            proc_macro_crate::FoundCrate::Name(name) => {
                let ident = syn::Ident::new(&name, Span::call_site());
                quote! { #ident }
            }
        };

        let self_cli_splice = self.cli.splice();
        parse_quote! {
            source_stream({
                use #root::util::cli::ConnectedSource;
                #self_cli_splice
                    .port(#port)
                    .connect_local_blocking::<#root::util::cli::ConnectedDirect>()
                    .into_source()
            })
        }
    }

    fn gen_sink_statement(&self, port: &String) -> Pipeline {
        let hydroflow_crate = proc_macro_crate::crate_name("hydroflow_plus")
            .expect("hydroflow_plus should be present in `Cargo.toml`");
        let root = match hydroflow_crate {
            proc_macro_crate::FoundCrate::Itself => quote! { hydroflow_plus },
            proc_macro_crate::FoundCrate::Name(name) => {
                let ident = syn::Ident::new(&name, Span::call_site());
                quote! { #ident }
            }
        };

        let self_cli_splice = self.cli.splice();
        parse_quote! {
            dest_sink({
                use #root::util::cli::ConnectedSink;
                #self_cli_splice
                    .port(#port)
                    .connect_local_blocking::<#root::util::cli::ConnectedDirect>()
                    .into_sink()
            })
        }
    }
}

pub trait HfSendTo<'a, O: HfNode<'a>>: HfNode<'a> {
    fn send_to(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);
}

impl<'a> HfSendTo<'a, CLIRuntimeNode<'a>> for CLIRuntimeNode<'a> {
    fn send_to(&self, _other: &CLIRuntimeNode, _source_port: &String, _recipient_port: &String) {}
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
    type NetworkedNode: HfNode<'a, Port = Self::Port> + HfSendTo<'a, Self::NetworkedNode>;
    type Port;
}

impl<'a, T: HfDeploy<'a, Node = N>, N: HfSendTo<'a, N>> HfNetworkedDeploy<'a> for T {
    type NetworkedNode = N;
    type Port = N::Port;
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
