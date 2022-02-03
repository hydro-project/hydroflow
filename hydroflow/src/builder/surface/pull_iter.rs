use super::{BaseSurface, PullSurface};

use crate::builder::build::pull_iter::IterPullBuild;

pub struct IterPullSurface<I, T>
where
    I: Iterator<Item = T>,
{
    it: I,
}

impl<I, T> IterPullSurface<I, T>
where
    I: Iterator<Item = T>,
{
    pub fn new(it: I) -> Self {
        Self { it }
    }
}

impl<I, T> BaseSurface for IterPullSurface<I, T>
where
    I: Iterator<Item = T>,
{
    type ItemOut = T;
}

impl<I, T> PullSurface for IterPullSurface<I, T>
where
    I: 'static + Iterator<Item = T>,
{
    type InputHandoffs = ();

    type Build = IterPullBuild<I, T>;

    fn into_build(self) -> Self::Build {
        IterPullBuild::new(self.it)
    }
}
