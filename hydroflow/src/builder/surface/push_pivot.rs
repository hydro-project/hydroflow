use super::{BaseSurface, PullSurface, PushSurface, PushSurfaceReversed};

use super::pivot::PivotSurface;

pub struct PivotPushSurface<Pull>
where
    Pull: PullSurface,
{
    pull: Pull,
}

impl<Pull> PivotPushSurface<Pull>
where
    Pull: PullSurface,
{
    pub fn new(pull: Pull) -> Self {
        Self { pull }
    }
}

impl<Pull> BaseSurface for PivotPushSurface<Pull>
where
    Pull: PullSurface,
{
    type ItemOut = Pull::ItemOut;
}

impl<Pull> PushSurface for PivotPushSurface<Pull>
where
    Pull: PullSurface,
{
    type Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    = PivotSurface<Pull, Next>;

    fn push_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    {
        PivotSurface::new(self.pull, next)
    }
}
