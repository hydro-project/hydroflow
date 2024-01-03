use std::cell::RefCell;

use stageleft::{Quoted, RuntimeData};

use super::{HfCluster, Location, LocalDeploy, ProcessSpec};
use crate::builder::Builders;
use crate::FlowBuilder;

pub struct SingleProcessGraph {}

impl<'a> LocalDeploy<'a> for SingleProcessGraph {
    type Process = SingleNode<'a>;
    type Cluster = SingleNode<'a>;
    type Meta = ();
    type RuntimeID = ();
}

impl<'a> ProcessSpec<'a, SingleProcessGraph> for () {
    fn build(
        &self,
        _id: usize,
        builder: &'a FlowBuilder<'a, SingleProcessGraph>,
        _meta: &mut (),
    ) -> SingleNode<'a> {
        SingleNode { builder }
    }
}

#[derive(Clone)]
pub struct SingleNode<'a> {
    builder: &'a FlowBuilder<'a, SingleProcessGraph>,
}

impl<'a> Location<'a> for SingleNode<'a> {
    type Port = ();
    type Meta = ();

    fn id(&self) -> usize {
        0
    }

    fn flow_builder(&self) -> (&'a RefCell<usize>, &'a Builders) {
        (&self.builder.next_id, &self.builder.builders)
    }

    fn next_port(&self) {
        panic!();
    }

    fn update_meta(&mut self, _meta: &Self::Meta) {}
}

impl<'a> HfCluster<'a> for SingleNode<'a> {
    fn ids(&self) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a {
        panic!();
        #[allow(unreachable_code)]
        RuntimeData::new("")
    }
}

pub struct MultiGraph {}

impl<'a> LocalDeploy<'a> for MultiGraph {
    type Process = MultiNode<'a>;
    type Cluster = MultiNode<'a>;
    type Meta = ();
    type RuntimeID = usize;
}

impl<'a> ProcessSpec<'a, MultiGraph> for () {
    fn build(
        &self,
        id: usize,
        builder: &'a FlowBuilder<'a, MultiGraph>,
        _meta: &mut (),
    ) -> MultiNode<'a> {
        MultiNode { builder, id }
    }
}

#[derive(Clone)]
pub struct MultiNode<'a> {
    builder: &'a FlowBuilder<'a, MultiGraph>,
    id: usize,
}

impl<'a> Location<'a> for MultiNode<'a> {
    type Port = ();
    type Meta = ();

    fn id(&self) -> usize {
        self.id
    }

    fn flow_builder(&self) -> (&'a RefCell<usize>, &'a Builders) {
        (&self.builder.next_id, &self.builder.builders)
    }

    fn next_port(&self) {
        panic!();
    }

    fn update_meta(&mut self, _meta: &Self::Meta) {}
}

impl<'a> HfCluster<'a> for MultiNode<'a> {
    fn ids(&self) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a {
        panic!();
        #[allow(unreachable_code)]
        RuntimeData::new("")
    }
}
