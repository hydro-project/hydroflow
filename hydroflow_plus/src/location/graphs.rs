use std::cell::RefCell;
use std::rc::Rc;

use stageleft::{Quoted, RuntimeData};

use super::{Cluster, LocalDeploy, Location, ProcessSpec};
use crate::ir::HfPlusLeaf;
use crate::FlowBuilder;

pub struct SingleProcessGraph {}

impl<'a> LocalDeploy<'a> for SingleProcessGraph {
    type ClusterID = ();
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
        SingleNode {
            builder,
            cycle_counter: Rc::new(RefCell::new(0)),
        }
    }
}

#[derive(Clone)]
pub struct SingleNode<'a> {
    builder: &'a FlowBuilder<'a, SingleProcessGraph>,
    cycle_counter: Rc<RefCell<usize>>,
}

impl<'a> Location<'a> for SingleNode<'a> {
    type Port = ();
    type Meta = ();

    fn id(&self) -> usize {
        0
    }

    fn ir_leaves(&self) -> &'a RefCell<Vec<HfPlusLeaf>> {
        self.builder.ir_leaves()
    }

    fn cycle_counter(&self) -> &RefCell<usize> {
        self.cycle_counter.as_ref()
    }

    fn next_port(&self) {
        panic!();
    }

    fn update_meta(&mut self, _meta: &Self::Meta) {}
}

impl<'a> Cluster<'a> for SingleNode<'a> {
    type Id = ();

    fn ids(&self) -> impl Quoted<'a, &'a Vec<()>> + Copy + 'a {
        panic!();
        #[allow(unreachable_code)]
        RuntimeData::new("")
    }
}

pub struct MultiGraph {}

impl<'a> LocalDeploy<'a> for MultiGraph {
    type ClusterID = u32;
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
        MultiNode {
            builder,
            id,
            cycle_counter: Rc::new(RefCell::new(0)),
        }
    }
}

#[derive(Clone)]
pub struct MultiNode<'a> {
    builder: &'a FlowBuilder<'a, MultiGraph>,
    id: usize,
    cycle_counter: Rc<RefCell<usize>>,
}

impl<'a> Location<'a> for MultiNode<'a> {
    type Port = ();
    type Meta = ();

    fn id(&self) -> usize {
        self.id
    }

    fn ir_leaves(&self) -> &'a RefCell<Vec<HfPlusLeaf>> {
        self.builder.ir_leaves()
    }

    fn cycle_counter(&self) -> &RefCell<usize> {
        self.cycle_counter.as_ref()
    }

    fn next_port(&self) {
        panic!();
    }

    fn update_meta(&mut self, _meta: &Self::Meta) {}
}

impl<'a> Cluster<'a> for MultiNode<'a> {
    type Id = u32;

    fn ids(&self) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a {
        panic!();
        #[allow(unreachable_code)]
        RuntimeData::new("")
    }
}
