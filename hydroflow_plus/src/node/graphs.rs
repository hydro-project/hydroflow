use hydroflow_lang::parse::Pipeline;

use super::{HfDeploy, HfNode, HfNodeBuilder};
use crate::HfBuilder;

pub struct SingleGraph {}

impl<'a> HfDeploy<'a> for SingleGraph {
    type Node = SingleNode<'a>;
}

impl<'a> HfNodeBuilder<'a, SingleGraph> for () {
    fn build(&mut self, _id: usize, builder: &'a HfBuilder<'a, SingleGraph>) -> SingleNode<'a> {
        SingleNode { builder }
    }
}

#[derive(Clone)]
pub struct SingleNode<'a> {
    builder: &'a HfBuilder<'a, SingleGraph>,
}

impl<'a> HfNode<'a, SingleGraph> for SingleNode<'a> {
    type Port = ();

    fn id(&self) -> usize {
        0
    }

    fn builder(&self) -> &'a HfBuilder<'a, SingleGraph> {
        self.builder
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

pub struct MultiGraph {}

impl<'a> HfDeploy<'a> for MultiGraph {
    type Node = MultiNode<'a>;
}

impl<'a> HfNodeBuilder<'a, MultiGraph> for () {
    fn build(&mut self, id: usize, builder: &'a HfBuilder<'a, MultiGraph>) -> MultiNode<'a> {
        MultiNode { builder, id }
    }
}

#[derive(Clone)]
pub struct MultiNode<'a> {
    builder: &'a HfBuilder<'a, MultiGraph>,
    id: usize,
}

impl<'a> HfNode<'a, MultiGraph> for MultiNode<'a> {
    type Port = ();

    fn id(&self) -> usize {
        self.id
    }

    fn builder(&self) -> &'a HfBuilder<'a, MultiGraph> {
        self.builder
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
