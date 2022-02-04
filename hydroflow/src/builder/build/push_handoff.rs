use super::{PushBuild, PushBuildBase};

use std::marker::PhantomData;

use crate::compiled::push_handoff::PushHandoff;
use crate::scheduled::handoff::handoff_list::PortList;
use crate::scheduled::handoff::{CanReceive, Handoff};
use crate::scheduled::port::{SendPort, SEND};
use crate::{tl, tt};

pub struct HandoffPushBuild<Hof, In>
where
    Hof: Handoff + CanReceive<In>,
{
    _phantom: PhantomData<fn(Hof, In)>,
}

impl<Hof, In> Default for HandoffPushBuild<Hof, In>
where
    Hof: Handoff + CanReceive<In>,
{
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<Hof, In> HandoffPushBuild<Hof, In>
where
    Hof: Handoff + CanReceive<In>,
{
    pub fn new() -> Self {
        Default::default()
    }
}

impl<Hof, In> PushBuildBase for HandoffPushBuild<Hof, In>
where
    Hof: Handoff + CanReceive<In>,
{
    type ItemIn = In;
    type Build<'slf, 'hof> = PushHandoff<'hof, Hof, In>;
}

impl<Hof, In> PushBuild for HandoffPushBuild<Hof, In>
where
    Hof: Handoff + CanReceive<In>,
{
    type OutputHandoffs = tt!(SendPort<Hof>);

    fn build<'slf, 'hof>(
        &'slf mut self,
        handoffs: <Self::OutputHandoffs as PortList<SEND>>::Ctx<'hof>,
    ) -> Self::Build<'slf, 'hof> {
        let tl!(handoff) = handoffs;
        PushHandoff::new(handoff)
    }
}
