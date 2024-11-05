use super::{Cluster, Location, LocationId, Process};
use crate::builder::FlowState;

pub trait NoTick {}
impl<T> NoTick for Process<'_, T> {}
impl<T> NoTick for Cluster<'_, T> {}

/// Marks the stream as being inside the single global clock domain.
#[derive(Clone)]
pub struct Tick<L> {
    pub(crate) l: L,
}

impl<'a, L: Location<'a>> Tick<L> {
    pub fn outer(&self) -> &L {
        &self.l
    }
}

impl<'a, L: Location<'a>> Location<'a> for Tick<L> {
    fn id(&self) -> LocationId {
        self.l.id()
    }

    fn flow_state(&self) -> &FlowState {
        self.l.flow_state()
    }

    fn is_top_level() -> bool {
        false
    }
}
