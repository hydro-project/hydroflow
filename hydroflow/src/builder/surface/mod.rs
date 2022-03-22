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
//!     * To switch to push, call [`PullSurface::pull_to_push`].
//! * [`PushSurface`] provides sink chaining methods and methods to split into multiple output streams: [`PushSurface::tee`], [`PushSurface::for_each`].
//!
//! For implementation info see [super].

use super::build::{PullBuild, PushBuild};

pub mod filter;
pub mod filter_map;
pub mod flatten;
pub mod map;
pub mod pivot;

pub mod pull_batch;
pub mod pull_chain;
pub mod pull_cross_join;
pub mod pull_half_hash_join;
pub mod pull_handoff;
pub mod pull_iter;
pub mod pull_join;

pub mod push_for_each;
pub mod push_handoff;
pub mod push_partition;
pub mod push_pivot;
pub mod push_start;
pub mod push_tee;

pub mod exchange;

use std::hash::Hash;

use crate::lang::lattice::{LatticeRepr, Merge};
use crate::scheduled::context::Context;
use crate::scheduled::graph::DirectedEdgeSet;
use crate::scheduled::handoff::handoff_list::{PortList, PortListSplit};
use crate::scheduled::port::{RECV, SEND};
use crate::scheduled::type_list::Extend;

pub trait TrackPushDependencies {
    fn insert_dep(&self, e: &mut DirectedEdgeSet) -> u16; // id of the inserted node
}
pub trait TrackPullDependencies {
    fn insert_dep(&self, e: &mut DirectedEdgeSet) -> u16; // id of the inserted node
}
/// Common trait shared between push and pull surface APIs.
///
/// Provides non-push/pull-specific chaining methods.
pub trait BaseSurface {
    type ItemOut;

    fn map_with_context<Func, Out>(self, func: Func) -> map::MapSurface<Self, Func>
    where
        Self: Sized,
        Func: FnMut(&Context<'_>, Self::ItemOut) -> Out,
    {
        map::MapSurface::new(self, func)
    }

    fn map<Func, Out>(self, mut func: Func) -> map::MapSurface<Self, MapNoCtxFn<Self, Func, Out>>
    where
        Self: Sized,
        Func: FnMut(Self::ItemOut) -> Out,
    {
        map::MapSurface::new(self, move |_ctx, x| (func)(x))
    }

    fn flat_map_with_context<Func, Out>(
        self,
        func: Func,
    ) -> flatten::FlattenSurface<map::MapSurface<Self, Func>>
    where
        Self: Sized,
        Func: FnMut(&Context<'_>, Self::ItemOut) -> Out,
        Out: IntoIterator,
    {
        self.map_with_context(func).flatten()
    }

    fn flat_map<Func, Out>(
        self,
        func: Func,
    ) -> flatten::FlattenSurface<map::MapSurface<Self, MapNoCtxFn<Self, Func, Out>>>
    where
        Self: Sized,
        Func: FnMut(Self::ItemOut) -> Out,
        Out: IntoIterator,
    {
        self.map(func).flatten()
    }

    fn flatten(self) -> flatten::FlattenSurface<Self>
    where
        Self: Sized,
        Self::ItemOut: IntoIterator,
    {
        flatten::FlattenSurface::new(self)
    }

    fn filter_with_context<Func>(self, func: Func) -> filter::FilterSurface<Self, Func>
    where
        Self: Sized,
        Func: FnMut(&Context<'_>, &Self::ItemOut) -> bool,
    {
        filter::FilterSurface::new(self, func)
    }

    fn filter<Func>(self, mut func: Func) -> filter::FilterSurface<Self, FilterNoCtxFn<Self, Func>>
    where
        Self: Sized,
        Func: FnMut(&Self::ItemOut) -> bool,
    {
        self.filter_with_context(move |_ctx, x| (func)(x))
    }

    fn filter_map_with_context<Func, Out>(
        self,
        func: Func,
    ) -> filter_map::FilterMapSurface<Self, Func>
    where
        Self: Sized,
        Func: FnMut(&Context<'_>, Self::ItemOut) -> Option<Out>,
    {
        filter_map::FilterMapSurface::new(self, func)
    }

    fn filter_map<Func, Out>(
        self,
        mut func: Func,
    ) -> filter_map::FilterMapSurface<Self, FilterMapNoCtxFn<Self, Func, Out>>
    where
        Self: Sized,
        Func: FnMut(Self::ItemOut) -> Option<Out>,
    {
        self.filter_map_with_context(move |_ctx, x| (func)(x))
    }

    fn map_scan<State, Func, Out>(
        self,
        mut initial_state: State,
        mut func: Func,
    ) -> map::MapSurface<Self, MapScanMapFunc<Self, State, Func, Out>>
    where
        Self: Sized,
        Func: FnMut(&mut State, Self::ItemOut) -> Out,
    {
        // TODO(mingwei): use state API.
        self.map_with_context(move |_ctx, item| func(&mut initial_state, item))
    }

    fn inspect_with_context<Func>(
        self,
        mut func: Func,
    ) -> map::MapSurface<Self, InspectMapFunc<Self, Func>>
    where
        Self: Sized,
        Func: FnMut(&Context<'_>, &Self::ItemOut),
    {
        self.map_with_context(move |context, item| {
            (func)(context, &item);
            item
        })
    }

    fn inspect<Func>(self, mut func: Func) -> map::MapSurface<Self, InspectMapNoCtxFunc<Self, Func>>
    where
        Self: Sized,
        Func: FnMut(&Self::ItemOut),
    {
        self.map_with_context(move |_ctx, item| {
            (func)(&item);
            item
        })
    }
}

pub type MapNoCtxFn<Prev, Func, Out>
where
    Prev: BaseSurface,
= impl FnMut(&Context<'_>, Prev::ItemOut) -> Out;

pub type FilterNoCtxFn<Prev, Func>
where
    Prev: BaseSurface,
= impl FnMut(&Context<'_>, &Prev::ItemOut) -> bool;

pub type FilterMapNoCtxFn<Prev, Func, Out>
where
    Prev: BaseSurface,
= impl FnMut(&Context<'_>, Prev::ItemOut) -> Option<Out>;

pub type MapScanMapFunc<Prev, State, Func, Out>
where
    Prev: BaseSurface,
= impl FnMut(&Context<'_>, Prev::ItemOut) -> Out;

pub type InspectMapFunc<Prev, Func>
where
    Prev: BaseSurface,
= impl FnMut(&Context<'_>, Prev::ItemOut) -> Prev::ItemOut;

pub type InspectMapNoCtxFunc<Prev, Func>
where
    Prev: BaseSurface,
= impl FnMut(&Context<'_>, Prev::ItemOut) -> Prev::ItemOut;

pub trait PullSurface: BaseSurface {
    type InputHandoffs: PortList<RECV>;
    type Build: PullBuild<InputHandoffs = Self::InputHandoffs, ItemOut = Self::ItemOut>;

    fn into_parts(self) -> (Self::InputHandoffs, Self::Build);

    fn chain<Other>(self, other: Other) -> pull_chain::ChainPullSurface<Self, Other>
    where
        Self: Sized,
        Other: PullSurface<ItemOut = Self::ItemOut>,

        Self::InputHandoffs: Extend<Other::InputHandoffs>,
        <Self::InputHandoffs as Extend<Other::InputHandoffs>>::Extended: PortList<RECV>
            + PortListSplit<RECV, Self::InputHandoffs, Suffix = Other::InputHandoffs>,
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

        Self::InputHandoffs: Extend<Other::InputHandoffs>,
        <Self::InputHandoffs as Extend<Other::InputHandoffs>>::Extended: PortList<RECV>
            + PortListSplit<RECV, Self::InputHandoffs, Suffix = Other::InputHandoffs>,
    {
        pull_join::JoinPullSurface::new(self, other)
    }

    fn batch_with<Other, L, Update, Tick>(
        self,
        other: Other,
    ) -> pull_batch::BatchPullSurface<Self, Other, L, Update, Tick>
    where
        Self: Sized + PullSurface<ItemOut = Update::Repr>,
        Other: PullSurface<ItemOut = Tick>,
        Update: 'static + LatticeRepr,
        L: 'static + LatticeRepr + Merge<Update>,

        Self::InputHandoffs: Extend<Other::InputHandoffs>,
        <Self::InputHandoffs as Extend<Other::InputHandoffs>>::Extended: PortList<RECV>
            + PortListSplit<RECV, Self::InputHandoffs, Suffix = Other::InputHandoffs>,
    {
        pull_batch::BatchPullSurface::new(self, other)
    }

    fn half_hash_join<Other, Key, ValSelf, L, Update>(
        self,
        other: Other,
    ) -> pull_half_hash_join::HalfHashJoinPullSurface<Self, Other, L, Update>
    where
        Self: Sized + PullSurface<ItemOut = (Key, ValSelf)>,
        Other: PullSurface<ItemOut = (Key, Update::Repr)>,
        Key: 'static + Eq + Hash,
        ValSelf: 'static,
        Update: 'static + LatticeRepr,
        L: 'static + LatticeRepr + Merge<Update>,
        L::Repr: Clone,

        Other::InputHandoffs: Extend<Self::InputHandoffs>,
        <Other::InputHandoffs as Extend<Self::InputHandoffs>>::Extended: PortList<RECV>
            + PortListSplit<RECV, Other::InputHandoffs, Suffix = Self::InputHandoffs>,
    {
        pull_half_hash_join::HalfHashJoinPullSurface::new(self, other)
    }

    fn cross_join<Other>(self, other: Other) -> pull_cross_join::CrossJoinPullSurface<Self, Other>
    where
        Self: Sized + PullSurface,
        Other: PullSurface,
        Self::ItemOut: 'static + Eq + Clone,
        Other::ItemOut: 'static + Eq + Clone,

        Self::InputHandoffs: Extend<Other::InputHandoffs>,
        <Self::InputHandoffs as Extend<Other::InputHandoffs>>::Extended: PortList<RECV>
            + PortListSplit<RECV, Self::InputHandoffs, Suffix = Other::InputHandoffs>,
    {
        pull_cross_join::CrossJoinPullSurface::new(self, other)
    }

    fn pull_to_push(self) -> push_pivot::PivotPushSurface<Self>
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

    fn push_to<Next>(self, next: Next) -> Self::Output<Next>
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
        <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended: PortList<SEND>
            + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
    {
        let next = push_tee::TeePushSurfaceReversed::new(next_a, next_b);
        self.push_to(next)
    }

    fn for_each_with_context<Func>(
        self,
        func: Func,
    ) -> Self::Output<push_for_each::ForEachPushSurfaceReversed<Func, Self::ItemOut>>
    where
        Self: Sized,
        Func: FnMut(&Context<'_>, Self::ItemOut),
    {
        let next = push_for_each::ForEachPushSurfaceReversed::new(func);
        self.push_to(next)
    }

    fn for_each<Func>(
        self,
        mut func: Func,
    ) -> Self::Output<
        push_for_each::ForEachPushSurfaceReversed<ForEachNoCtxFunc<Self, Func>, Self::ItemOut>,
    >
    where
        Self: Sized,
        Func: FnMut(Self::ItemOut),
    {
        self.for_each_with_context(move |_ctx, x| (func)(x))
    }

    fn partition_with_context<Func, NextA, NextB>(
        self,
        func: Func,
        next_a: NextA,
        next_b: NextB,
    ) -> Self::Output<push_partition::PartitionPushSurfaceReversed<NextA, NextB, Func>>
    where
        Self: Sized,
        Func: FnMut(&Context<'_>, &Self::ItemOut) -> bool,
        NextA: PushSurfaceReversed<ItemIn = Self::ItemOut>,
        NextB: PushSurfaceReversed<ItemIn = Self::ItemOut>,

        NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
        <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended: PortList<SEND>
            + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
    {
        let next = push_partition::PartitionPushSurfaceReversed::new(func, next_a, next_b);
        self.push_to(next)
    }

    fn partition<Func, NextA, NextB>(
        self,
        func: Func,
        next_a: NextA,
        next_b: NextB,
    ) -> Self::Output<PartitionNoCtxOutput<Self, Func, NextA, NextB>>
    where
        Self: Sized,
        Func: Fn(&Self::ItemOut) -> bool,
        NextA: PushSurfaceReversed<ItemIn = Self::ItemOut>,
        NextB: PushSurfaceReversed<ItemIn = Self::ItemOut>,

        NextA::OutputHandoffs: Extend<NextB::OutputHandoffs>,
        <NextA::OutputHandoffs as Extend<NextB::OutputHandoffs>>::Extended: PortList<SEND>
            + PortListSplit<SEND, NextA::OutputHandoffs, Suffix = NextB::OutputHandoffs>,
    {
        self.partition_with_context(move |_ctx, x| (func)(x), next_a, next_b)
    }
}

pub type ForEachNoCtxFunc<Prev, Func>
where
    Prev: BaseSurface,
= impl FnMut(&Context<'_>, Prev::ItemOut);

pub type PartitionNoCtxOutput<Prev, Func, NextA, NextB>
where
    Prev: BaseSurface,
= push_partition::PartitionPushSurfaceReversed<
    NextA,
    NextB,
    impl FnMut(&Context<'_>, &Prev::ItemOut) -> bool,
>;

/// This extra layer is needed due to the ownership order. In the functional
/// chaining syntax each operator owns the previous (can only go in order
/// things are called/defined), but in the end we need each pusherator to own
/// the _next_ pusherator which it's pushing to.
///
/// This is the already-reversed, [PushSurface] does the actual reversing.
pub trait PushSurfaceReversed {
    type ItemIn;

    type OutputHandoffs: PortList<SEND>;
    type Build: PushBuild<OutputHandoffs = Self::OutputHandoffs, ItemIn = Self::ItemIn>;

    fn into_parts(self) -> (Self::OutputHandoffs, Self::Build);
}
