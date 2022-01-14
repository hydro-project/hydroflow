//! Structs used to create the user-facing Surface API.
//!
//! Main user-facing traits are [`BaseSurface`], [`PullSurface`], and
//! [`PushSurface`], which provide an iterator-like API with easy method
//! chaining. The traits need to be imported when using the Surface API.
//! You can use the prelude to do this easily:
//! ```ignore
//! use hydroflow::build::prelude::*;
//! ```
//!
//! * [`BaseSurface`] provides linear chaining methods like [`BaseSurface::map`], [`BaseSurface::filter`], etc..
//! * [`PullSurface`] provides methods to combine multiple input streams: [`PullSurface::chain`], [`PullSurface::join`].
//!     * To switch to push, call [`PullSurface::pivot`].
//! * [`PushSurface`] provides sink chaining methods and methods to split into multiple output streams: [`PushSurface::tee`], [`PushSurface::for_each`].
//!
//! For implementation info see [super].

use super::build::{PullBuild, PushBuild};
use super::connect::{PullConnect, PushConnect};

pub mod filter;
pub mod flat_map;
pub mod map;
pub mod pivot;

pub mod pull_chain;
pub mod pull_handoff;
pub mod pull_join;
pub mod pull_ripple_join;

pub mod push_for_each;
pub mod push_handoff;
pub mod push_pivot;
pub mod push_start;
pub mod push_tee;

use std::hash::Hash;

use crate::scheduled::handoff::{HandoffList, HandoffListSplit};
use crate::scheduled::type_list::Extend;

/// Common trait shared between push and pull surface APIs.
///
/// Provides non-push/pull-specific chaining methods.
pub trait BaseSurface {
    type ItemOut;

    fn map<Func, Out>(self, func: Func) -> map::MapSurface<Self, Func>
    where
        Self: Sized,
        Func: FnMut(Self::ItemOut) -> Out,
    {
        map::MapSurface::new(self, func)
    }

    fn flat_map<Func, Out>(self, func: Func) -> flat_map::FlatMapSurface<Self, Func>
    where
        Self: Sized,
        Func: FnMut(Self::ItemOut) -> Out,
        Out: IntoIterator,
    {
        flat_map::FlatMapSurface::new(self, func)
    }

    fn filter<Func>(self, func: Func) -> filter::FilterSurface<Self, Func>
    where
        Self: Sized,
        Func: FnMut(&Self::ItemOut) -> bool,
    {
        filter::FilterSurface::new(self, func)
    }
}

pub trait PullSurface: BaseSurface {
    type InputHandoffs: HandoffList;

    type Connect: PullConnect<InputHandoffs = Self::InputHandoffs>;
    type Build: PullBuild<InputHandoffs = Self::InputHandoffs, ItemOut = Self::ItemOut>;

    fn into_parts(self) -> (Self::Connect, Self::Build);

    fn chain<Other>(self, other: Other) -> pull_chain::ChainPullSurface<Self, Other>
    where
        Self: Sized,
        Other: PullSurface<ItemOut = Self::ItemOut>,
    {
        pull_chain::ChainPullSurface::new(self, other)
    }

    fn join<Other, Key, ValSelf, ValOther>(
        self,
        other: Other,
    ) -> pull_join::JoinPullSurface<Self, Other>
    where
        Self: Sized + PullSurface<ItemOut = (Key, ValSelf)>,
        Other: PullSurface<ItemOut = (Key, ValOther)>,
        Key: 'static + Eq + Hash + Clone,
        ValSelf: 'static + Eq + Clone,
        ValOther: 'static + Eq + Clone,
    {
        pull_join::JoinPullSurface::new(self, other)
    }

    fn ripple_join<Other>(
        self,
        other: Other,
    ) -> pull_ripple_join::RippleJoinPullSurface<Self, Other>
    where
        Self: Sized + PullSurface,
        Other: PullSurface,
        Self::ItemOut: 'static + Eq + Clone,
        Other::ItemOut: 'static + Eq + Clone,
    {
        pull_ripple_join::RippleJoinPullSurface::new(self, other)
    }

    fn pivot(self) -> push_pivot::PivotPushSurface<Self>
    where
        Self: Sized,
    {
        push_pivot::PivotPushSurface::new(self)
    }
}

pub trait PushSurface: BaseSurface {
    /// This should usually be a type which impls [PushSurfaceReversed], but it is not enforced since we also need to return a Pivot in the end.
    type Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>;

    fn reverse<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>;

    /// To create a output tee, use [`HydroflowBuilder::start_tee()`](crate::builder::HydroflowBuilder::start_tee).
    fn tee<NextA, NextB>(
        self,
        next_a: NextA,
        next_b: NextB,
    ) -> Self::Output<push_tee::TeePushSurfaceReversed<NextA, NextB>>
    where
        Self: Sized,
        Self::ItemOut: Clone,
        NextA: PushSurfaceReversed<ItemIn = Self::ItemOut>,
        NextB: PushSurfaceReversed<ItemIn = Self::ItemOut>,

        NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
        <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended:
            HandoffList + HandoffListSplit<NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
    {
        let next = push_tee::TeePushSurfaceReversed::new(next_a, next_b);
        self.reverse(next)
    }

    fn for_each<Func>(
        self,
        func: Func,
    ) -> Self::Output<push_for_each::ForEachPushSurfaceReversed<Func, Self::ItemOut>>
    where
        Self: Sized,
        Func: FnMut(Self::ItemOut),
    {
        let next = push_for_each::ForEachPushSurfaceReversed::new(func);
        self.reverse(next)
    }
}

/// This extra layer is needed due to the ownership order. In the functional
/// chaining syntax each operator owns the previous (can only go in order
/// things are called/defined), but in the end we need each pusherator to own
/// the _next_ pusherator which it's pushing to.
///
/// This is the already-reversed, [PushSurface] does the actual reversing.
pub trait PushSurfaceReversed {
    type OutputHandoffs: HandoffList;

    type ItemIn;

    type Connect: PushConnect<OutputHandoffs = Self::OutputHandoffs>;
    type Build: PushBuild<OutputHandoffs = Self::OutputHandoffs, ItemIn = Self::ItemIn>;

    fn into_parts(self) -> (Self::Connect, Self::Build);
}
