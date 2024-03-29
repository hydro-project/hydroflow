use std::cell::RefCell;
use std::rc::Rc;

use stageleft::{Quoted, RuntimeData};

use super::{Cluster, LocalDeploy, Location, ProcessSpec};
use crate::ir::HfPlusLeaf;
use crate::FlowBuilder;

pub struct SingleProcessGraph {}

impl<'a> LocalDeploy<'a> for SingleProcessGraph {
    type ClusterId = ();
    type Process = SingleNode;
    type Cluster = SingleNode;
    type Meta = ();
    type GraphId = ();
}

impl<'a> ProcessSpec<'a, SingleProcessGraph> for () {
    fn build(
        &self,
        _id: usize,
        builder: &FlowBuilder<'a, SingleProcessGraph>,
        _meta: &mut (),
    ) -> SingleNode {
        SingleNode {
            ir_leaves: builder.ir_leaves().clone(),
            cycle_counter: Rc::new(RefCell::new(0)),
        }
    }
}

#[derive(Clone)]
pub struct SingleNode {
    ir_leaves: Rc<RefCell<Vec<HfPlusLeaf>>>,
    cycle_counter: Rc<RefCell<usize>>,
}

impl<'a> Location<'a> for SingleNode {
    type Port = ();
    type Meta = ();

    fn id(&self) -> usize {
        0
    }

    fn ir_leaves(&self) -> &Rc<RefCell<Vec<HfPlusLeaf>>> {
        &self.ir_leaves
    }

    fn cycle_counter(&self) -> &RefCell<usize> {
        self.cycle_counter.as_ref()
    }

    fn next_port(&self) {
        panic!();
    }

    fn update_meta(&mut self, _meta: &Self::Meta) {}
}

impl<'a> Cluster<'a> for SingleNode {
    type Id = ();

    fn ids(&self) -> impl Quoted<'a, &'a Vec<()>> + Copy + 'a {
        panic!();
        #[allow(unreachable_code)]
        RuntimeData::new("")
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
    fn build(&self, id: usize, builder: &FlowBuilder<'a, MultiGraph>, _meta: &mut ()) -> MultiNode {
        MultiNode {
            ir_leaves: builder.ir_leaves().clone(),
            id,
            cycle_counter: Rc::new(RefCell::new(0)),
        }
    }
}

#[derive(Clone)]
pub struct MultiNode {
    ir_leaves: Rc<RefCell<Vec<HfPlusLeaf>>>,
    id: usize,
    cycle_counter: Rc<RefCell<usize>>,
}

impl<'a> Location<'a> for MultiNode {
    type Port = ();
    type Meta = ();

    fn id(&self) -> usize {
        self.id
    }

    fn ir_leaves(&self) -> &Rc<RefCell<Vec<HfPlusLeaf>>> {
        &self.ir_leaves
    }

    fn cycle_counter(&self) -> &RefCell<usize> {
        self.cycle_counter.as_ref()
    }

    fn next_port(&self) {
        panic!();
    }

    fn update_meta(&mut self, _meta: &Self::Meta) {}
}

impl<'a> Cluster<'a> for MultiNode {
    type Id = u32;

    fn ids(&self) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a {
        panic!();
        #[allow(unreachable_code)]
        RuntimeData::new("")
    }
}
