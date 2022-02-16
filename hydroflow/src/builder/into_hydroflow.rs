use super::surface::pull_iter::IterPullSurface;

/// A trait to convert [`IntoIterator`]s for use with Hydroflow's surface (build) API.
pub trait IntoHydroflow: IntoIterator {
    /// Convert this [`IntoIterator`] into a hydroflow subgraph builder.
    fn into_hydroflow(self) -> IterPullSurface<Self::IntoIter>
    where
        Self: Sized,
    {
        IterPullSurface::new(self.into_iter())
    }
}
impl<I> IntoHydroflow for I where I: IntoIterator {}
