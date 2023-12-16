use std::cell::RefCell;
use std::rc::Rc;

use hydroflow_plus::lang::parse::Pipeline;
use hydroflow_plus::node::{HfDeploy, HfNode, HfNodeBuilder, HfSendTo};
use hydroflow_plus::util::cli::HydroCLI;
use hydroflow_plus::HfBuilder;
use stageleft::internal::{quote, Span};
use stageleft::{Quoted, RuntimeData};
use syn::parse_quote;

pub struct CLIRuntime {}

impl<'a> HfDeploy<'a> for CLIRuntime {
    type Node = CLIRuntimeNode<'a>;
}

#[derive(Clone)]
pub struct CLIRuntimeNode<'a> {
    id: usize,
    builder: &'a HfBuilder<'a, CLIRuntime>,
    next_port: Rc<RefCell<usize>>,
    cli: RuntimeData<&'a HydroCLI>,
}

impl<'a> HfNode<'a, CLIRuntime> for CLIRuntimeNode<'a> {
    type Port = String;

    fn id(&self) -> usize {
        self.id
    }

    fn builder(&self) -> &'a HfBuilder<'a, CLIRuntime> {
        self.builder
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

impl<'a> HfSendTo<'a, CLIRuntime, CLIRuntimeNode<'a>> for CLIRuntimeNode<'a> {
    fn send_to(&self, _other: &CLIRuntimeNode, _source_port: &String, _recipient_port: &String) {}
}

pub struct CLIRuntimeNodeBuilder<'a> {
    cli: RuntimeData<&'a HydroCLI>,
}

impl CLIRuntimeNodeBuilder<'_> {
    pub fn new(cli: RuntimeData<&HydroCLI>) -> CLIRuntimeNodeBuilder {
        CLIRuntimeNodeBuilder { cli }
    }
}

impl<'cli> HfNodeBuilder<'cli, CLIRuntime> for CLIRuntimeNodeBuilder<'cli> {
    fn build(
        &mut self,
        id: usize,
        builder: &'cli HfBuilder<'cli, CLIRuntime>,
    ) -> CLIRuntimeNode<'cli> {
        CLIRuntimeNode {
            id,
            builder,
            next_port: Rc::new(RefCell::new(0)),
            cli: self.cli,
        }
    }
}
