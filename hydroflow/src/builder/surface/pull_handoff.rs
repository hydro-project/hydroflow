use super::{AssembleFlowGraph, BaseSurface, PullSurface};

use crate::builder::build::pull_handoff::HandoffPullBuild;
use crate::scheduled::flow_graph::NodeId;
use crate::scheduled::handoff::Handoff;
use crate::scheduled::port::RecvPort;
use crate::{tl, tt};

pub struct HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    port: RecvPort<Hof>,
}

impl<Hof> HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    pub fn new(port: RecvPort<Hof>) -> Self {
        Self { port }
    }
}
impl<Hof> AssembleFlowGraph for HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> NodeId {
        let my_id = e.add_node("Handoff");
        e.add_handoff_id(my_id, self.port.handoff_id);
        my_id
    }
}

impl<Hof> BaseSurface for HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    type ItemOut = Hof::Inner;
}

impl<Hof> PullSurface for HandoffPullSurface<Hof>
where
    Hof: Handoff,
{
    type InputHandoffs = tt!(RecvPort<Hof>);
    type Build = HandoffPullBuild<Hof>;

    fn into_parts(self) -> (Self::InputHandoffs, Self::Build) {
        (tl!(self.port), HandoffPullBuild::new())
    }
}
