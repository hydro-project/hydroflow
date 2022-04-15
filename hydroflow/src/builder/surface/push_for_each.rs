use super::{AssembleFlowGraph, PushSurfaceReversed};

use std::marker::PhantomData;

use crate::builder::build::push_for_each::ForEachPushBuild;
use crate::scheduled::context::Context;
use crate::scheduled::flow_graph::NodeId;

pub struct ForEachPushSurfaceReversed<Func, In>
where
    Func: FnMut(&Context, In),
{
    func: Func,
    _phantom: PhantomData<fn(In)>,
}
impl<Func, In> ForEachPushSurfaceReversed<Func, In>
where
    Func: FnMut(&Context, In),
{
    pub fn new(func: Func) -> Self {
        Self {
            func,
            _phantom: PhantomData,
        }
    }
}
impl<Func, In> AssembleFlowGraph for ForEachPushSurfaceReversed<Func, In>
where
    Func: FnMut(&Context, In),
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> NodeId {
        let my_id = e.add_node("ForEach");
        my_id
    }
}

impl<Func, In> PushSurfaceReversed for ForEachPushSurfaceReversed<Func, In>
where
    Func: FnMut(&Context, In),
{
    type ItemIn = In;

    type OutputHandoffs = ();
    type Build = ForEachPushBuild<Func, In>;

    fn make_parts(self, _ctx: &mut Context) -> (Self::OutputHandoffs, Self::Build) {
        ((), ForEachPushBuild::new(self.func))
    }
}
