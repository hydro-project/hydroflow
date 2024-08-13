use hydroflow_lang::graph::HydroflowGraph;

use super::{LocalDeploy, Node, ProcessSpec};

pub struct SingleProcessGraph {}

impl<'a> LocalDeploy<'a> for SingleProcessGraph {
    type ClusterId = ();
    type Process = SingleNode;
    type Cluster = SingleNode;
    type Meta = ();
    type GraphId = ();
}

impl<'a> ProcessSpec<'a, SingleProcessGraph> for () {
    fn build(self, _id: usize) -> SingleNode {
        SingleNode {}
    }
}

#[derive(Clone)]
pub struct SingleNode {}

impl Node for SingleNode {
    type Port = ();
    type Meta = ();
    type InstantiateEnv = ();

    fn id(&self) -> usize {
        0
    }

    fn next_port(&self) {
        panic!();
    }

    fn update_meta(&mut self, _meta: &Self::Meta) {}

    fn instantiate(
        &self,
        _env: &mut Self::InstantiateEnv,
        _meta: &mut Self::Meta,
        _graph: HydroflowGraph,
    ) {
    }
}

pub struct MultiGraph {}

impl<'a> LocalDeploy<'a> for MultiGraph {
    type ClusterId = u32;
    type Process = MultiNode;
    type Cluster = MultiNode;
    type Meta = ();
    type GraphId = usize;
}

impl<'a> ProcessSpec<'a, MultiGraph> for () {
    fn build(self, id: usize) -> MultiNode {
        MultiNode { id }
    }
}

#[derive(Clone)]
pub struct MultiNode {
    id: usize,
}

impl Node for MultiNode {
    type Port = ();
    type Meta = ();
    type InstantiateEnv = ();

    fn id(&self) -> usize {
        self.id
    }

    fn next_port(&self) {
        panic!();
    }

    fn update_meta(&mut self, _meta: &Self::Meta) {}

    fn instantiate(
        &self,
        _env: &mut Self::InstantiateEnv,
        _meta: &mut Self::Meta,
        _graph: HydroflowGraph,
    ) {
    }
}
