use std::cell::RefCell;
use std::rc::Rc;

use hydroflow_plus::lang::parse::Pipeline;
use hydroflow_plus::node::{
    HfCluster, HfClusterBuilder, HfDeploy, HfNode, HfNodeBuilder, HfSendManyToMany,
    HfSendManyToOne, HfSendOneToMany, HfSendOneToOne,
};
use hydroflow_plus::util::cli::{
    ConnectedDemux, ConnectedDirect, ConnectedSink, ConnectedSource, ConnectedTagged, HydroCLI,
};
use hydroflow_plus::HfBuilder;
use stageleft::{q, Quoted, RuntimeData};
use syn::parse_quote;

use super::HydroflowPlusMeta;

pub struct CLIRuntime {}

impl<'a> HfDeploy<'a> for CLIRuntime {
    type Node = CLIRuntimeNode<'a>;
    type Cluster = CLIRuntimeCluster<'a>;
    type Meta = String;
    type RuntimeID = usize;
}

#[derive(Clone)]
pub struct CLIRuntimeNode<'a> {
    id: usize,
    builder: &'a HfBuilder<'a, CLIRuntime>,
    next_port: Rc<RefCell<usize>>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
}

impl<'a> HfNode<'a> for CLIRuntimeNode<'a> {
    type Port = String;
    type Meta = String;

    fn id(&self) -> usize {
        self.id
    }

    fn graph_builder(&self) -> (&'a RefCell<usize>, &'a hydroflow_plus::builder::Builders) {
        self.builder.builder_components()
    }

    fn next_port(&self) -> String {
        let next_send_port = *self.next_port.borrow();
        *self.next_port.borrow_mut() += 1;
        format!("port_{}", next_send_port)
    }

    fn build(&mut self, _meta: &Option<Self::Meta>) {}
}

#[derive(Clone)]
pub struct CLIRuntimeCluster<'a> {
    id: usize,
    builder: &'a HfBuilder<'a, CLIRuntime>,
    next_port: Rc<RefCell<usize>>,
    cli: RuntimeData<&'a HydroCLI<HydroflowPlusMeta>>,
}

impl<'a> HfNode<'a> for CLIRuntimeCluster<'a> {
    type Port = String;
    type Meta = String;

    fn id(&self) -> usize {
        self.id
    }

    fn graph_builder(&self) -> (&'a RefCell<usize>, &'a hydroflow_plus::builder::Builders) {
        self.builder.builder_components()
    }

    fn next_port(&self) -> String {
        let next_send_port = *self.next_port.borrow();
        *self.next_port.borrow_mut() += 1;
        format!("port_{}", next_send_port)
    }

    fn build(&mut self, _meta: &Option<Self::Meta>) {}
}

impl<'a> HfCluster<'a> for CLIRuntimeCluster<'a> {
    fn ids(&self) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a {
        let cli = self.cli;
        let self_id = self.id;
        q!(cli.meta.as_ref().unwrap().clusters.get(&self_id).unwrap())
    }
}

impl<'a> HfSendOneToOne<'a, CLIRuntimeNode<'a>> for CLIRuntimeNode<'a> {
    fn connect(&self, _other: &CLIRuntimeNode, _source_port: &String, _recipient_port: &String) {}

    fn gen_sink_statement(&self, port: &String) -> Pipeline {
        let self_cli = self.cli;
        let port = port.as_str();
        let sink_quote = q!({
            self_cli
                .port(port)
                .connect_local_blocking::<ConnectedDirect>()
                .into_sink()
        })
        .splice();

        parse_quote!(dest_sink(#sink_quote))
    }

    fn gen_source_statement(other: &CLIRuntimeNode<'a>, port: &String) -> Pipeline {
        let self_cli = other.cli;
        let port = port.as_str();
        let source_quote = q!({
            self_cli
                .port(port)
                .connect_local_blocking::<ConnectedDirect>()
                .into_source()
        })
        .splice();

        parse_quote!(source_stream(#source_quote))
    }
}

impl<'a> HfSendManyToOne<'a, CLIRuntimeNode<'a>> for CLIRuntimeCluster<'a> {
    fn connect(&self, _other: &CLIRuntimeNode, _source_port: &String, _recipient_port: &String) {}

    fn gen_sink_statement(&self, port: &String) -> Pipeline {
        let self_cli = self.cli;
        let port = port.as_str();
        let sink_quote = q!({
            self_cli
                .port(port)
                .connect_local_blocking::<ConnectedDirect>()
                .into_sink()
        })
        .splice();

        parse_quote!(dest_sink(#sink_quote))
    }

    fn gen_source_statement(other: &CLIRuntimeNode<'a>, port: &String) -> Pipeline {
        let self_cli = other.cli;
        let port = port.as_str();
        let source_quote = q!({
            self_cli
                .port(port)
                .connect_local_blocking::<ConnectedTagged<ConnectedDirect>>()
                .into_source()
        })
        .splice();

        parse_quote!(source_stream(#source_quote))
    }
}

impl<'a> HfSendOneToMany<'a, CLIRuntimeCluster<'a>> for CLIRuntimeNode<'a> {
    fn connect(&self, _other: &CLIRuntimeCluster, _source_port: &String, _recipient_port: &String) {
    }

    fn gen_sink_statement(&self, port: &String) -> Pipeline {
        let self_cli = self.cli;
        let port = port.as_str();

        let sink_quote = q!({
            self_cli
                .port(port)
                .connect_local_blocking::<ConnectedDemux<ConnectedDirect>>()
                .into_sink()
        })
        .splice();

        parse_quote!(dest_sink(#sink_quote))
    }

    fn gen_source_statement(other: &CLIRuntimeCluster<'a>, port: &String) -> Pipeline {
        let self_cli = other.cli;
        let port = port.as_str();

        let source_quote = q!({
            self_cli
                .port(port)
                .connect_local_blocking::<ConnectedDirect>()
                .into_source()
        })
        .splice();

        parse_quote!(source_stream(#source_quote))
    }
}

impl<'a> HfSendManyToMany<'a, CLIRuntimeCluster<'a>> for CLIRuntimeCluster<'a> {
    fn connect(&self, _other: &CLIRuntimeCluster, _source_port: &String, _recipient_port: &String) {
    }

    fn gen_sink_statement(&self, port: &String) -> Pipeline {
        let self_cli = self.cli;
        let port = port.as_str();

        let sink_quote = q!({
            self_cli
                .port(port)
                .connect_local_blocking::<ConnectedDemux<ConnectedDirect>>()
                .into_sink()
        })
        .splice();

        parse_quote!(dest_sink(#sink_quote))
    }

    fn gen_source_statement(other: &CLIRuntimeCluster<'a>, port: &String) -> Pipeline {
        let self_cli = other.cli;
        let port = port.as_str();

        let source_quote = q!({
            self_cli
                .port(port)
                .connect_local_blocking::<ConnectedTagged<ConnectedDirect>>()
                .into_source()
        })
        .splice();

        parse_quote!(source_stream(#source_quote))
    }
}

impl<'cli> HfNodeBuilder<'cli, CLIRuntime> for RuntimeData<&'cli HydroCLI<HydroflowPlusMeta>> {
    fn build(&self, id: usize, builder: &'cli HfBuilder<'cli, CLIRuntime>) -> CLIRuntimeNode<'cli> {
        CLIRuntimeNode {
            id,
            builder,
            next_port: Rc::new(RefCell::new(0)),
            cli: *self,
        }
    }
}

impl<'cli> HfClusterBuilder<'cli, CLIRuntime> for RuntimeData<&'cli HydroCLI<HydroflowPlusMeta>> {
    fn build(
        &self,
        id: usize,
        builder: &'cli HfBuilder<'cli, CLIRuntime>,
    ) -> CLIRuntimeCluster<'cli> {
        CLIRuntimeCluster {
            id,
            builder,
            next_port: Rc::new(RefCell::new(0)),
            cli: *self,
        }
    }
}
