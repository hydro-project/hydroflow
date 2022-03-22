use super::{PushSurfaceReversed, TrackPushDependencies};

use std::marker::PhantomData;

use crate::{builder::build::push_for_each::ForEachPushBuild, scheduled::context::Context};

pub struct ForEachPushSurfaceReversed<Func, In>
where
    Func: FnMut(&Context<'_>, In),
{
    func: Func,
    _phantom: PhantomData<fn(In)>,
}
impl<Func, In> ForEachPushSurfaceReversed<Func, In>
where
    Func: FnMut(&Context<'_>, In),
{
    pub fn new(func: Func) -> Self {
        Self {
            func,
            _phantom: PhantomData,
        }
    }
}
impl<Func, In> TrackPushDependencies for ForEachPushSurfaceReversed<Func, In>
where
    Func: FnMut(In),
{
    fn insert_dep(&self, e: &mut super::DirectedEdgeSet) -> u16 {
        let my_id = e.add_node("ForEach".to_string());
        my_id
    }
}

impl<Func, In> PushSurfaceReversed for ForEachPushSurfaceReversed<Func, In>
where
    Func: FnMut(&Context<'_>, In),
{
    type ItemIn = In;

    type OutputHandoffs = ();
    type Build = ForEachPushBuild<Func, In>;

    fn into_parts(self) -> (Self::OutputHandoffs, Self::Build) {
        ((), ForEachPushBuild::new(self.func))
    }
}
