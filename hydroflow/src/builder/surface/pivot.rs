use super::{PullSurface, PushSurfaceReversed};

#[allow(type_alias_bounds)]
type Parts<Pull, Push>
where
    Pull: PullSurface,
    Push: PushSurfaceReversed<ItemIn = Pull::ItemOut>,
= (
    (Pull::InputHandoffs, Push::OutputHandoffs),
    (Pull::Build, Push::Build),
);

/// The combination of both Pull and Push surface halves.
pub struct PivotSurface<Pull, Push>
where
    Pull: PullSurface,
    Push: PushSurfaceReversed<ItemIn = Pull::ItemOut>,
{
    pub(crate) pull: Pull,
    pub(crate) push: Push,
}
impl<Pull, Push> PivotSurface<Pull, Push>
where
    Pull: PullSurface,
    Push: PushSurfaceReversed<ItemIn = Pull::ItemOut>,
{
    pub fn new(pull: Pull, push: Push) -> Self {
        Self { pull, push }
    }

    pub fn into_parts(self) -> Parts<Pull, Push> {
        let (pull_connect, pull_build) = self.pull.into_parts();
        let (push_connect, push_build) = self.push.into_parts();
        ((pull_connect, push_connect), (pull_build, push_build))
    }
}
