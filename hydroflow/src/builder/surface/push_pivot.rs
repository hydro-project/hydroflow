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

// impl<Pull> TrackPullDependencies for PivotPushSurface<Pull>
// where
//     Pull: PullSurface + TrackPullDependencies,
// {
//     fn insert_dep(&self, e: &mut super::DirectedEdgeSet) -> u16 {
//         let my_id = e.add_node("PivotPush".to_string());
//         let pull_id = self.pull.insert_dep(e);
//         e.add_edge((pull_id, my_id));
//         my_id
//     }
// }

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
    type Output<Next> = PivotSurface<Pull, Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>;

    fn push_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    {
        PivotSurface::new(self.pull, next)
    }
}
