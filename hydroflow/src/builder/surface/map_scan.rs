use super::{AssembleFlowGraph, BaseSurface, PullSurface, PushSurface, PushSurfaceReversed};

use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;

use crate::builder::build::pull_map::MapPullBuild;
use crate::builder::build::push_map::MapPushBuild;
use crate::scheduled::context::Context;
use crate::scheduled::flow_graph::FlowNodeId;
use crate::scheduled::state::StateHandle;

type MapScanFunc<Func, State, In, Out>
where
    Func: FnMut(&mut State, In) -> Out,
    State: Any,
= impl FnMut(&Context, In) -> Out;

fn wrap_func<Func, State, In, Out>(
    mut func: Func,
    state_handle: StateHandle<RefCell<State>>,
) -> MapScanFunc<Func, State, In, Out>
where
    Func: FnMut(&mut State, In) -> Out,
    State: Any,
{
    move |context, x| {
        let mut state = context.state_ref(state_handle).borrow_mut();
        (func)(&mut *state, x)
    }
}

pub struct MapScanSurface<Prev, Func, State>
where
    Prev: BaseSurface,
    State: Any,
{
    prev: Prev,
    func: Func,
    state: State,
}
impl<Prev, Func, State, Out> MapScanSurface<Prev, Func, State>
where
    Prev: BaseSurface,
    Func: FnMut(&mut State, Prev::ItemOut) -> Out,
    State: Any,
{
    pub fn new(prev: Prev, func: Func, state: State) -> Self {
        Self { prev, func, state }
    }
}

impl<Prev, Func, State, Out> BaseSurface for MapScanSurface<Prev, Func, State>
where
    Prev: BaseSurface,
    Func: FnMut(&mut State, Prev::ItemOut) -> Out,
    State: Any,
{
    type ItemOut = Out;
}

impl<Prev, Func, State, Out> PullSurface for MapScanSurface<Prev, Func, State>
where
    Prev: PullSurface,
    Func: FnMut(&mut State, Prev::ItemOut) -> Out,
    State: Any,
{
    type InputHandoffs = Prev::InputHandoffs;
    type Build = MapPullBuild<Prev::Build, MapScanFunc<Func, State, Prev::ItemOut, Out>>;

    fn make_parts(self, ctx: &mut Context) -> (Self::InputHandoffs, Self::Build) {
        let state_handle = ctx.add_state(RefCell::new(self.state));

        let (connect, build) = self.prev.make_parts(ctx);
        let build = MapPullBuild::new(build, wrap_func(self.func, state_handle));
        (connect, build)
    }
}
impl<Prev, Func, State, Out> AssembleFlowGraph for MapScanSurface<Prev, Func, State>
where
    Prev: PullSurface + AssembleFlowGraph,
    Func: FnMut(&mut State, Prev::ItemOut) -> Out,
    State: Any,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> FlowNodeId {
        let my_id = e.add_node("MapScan");
        let prev_id = self.prev.insert_dep(e);
        e.add_edge((prev_id, my_id));
        my_id
    }
}

impl<Prev, Func, State, Out> PushSurface for MapScanSurface<Prev, Func, State>
where
    Prev: PushSurface,
    Func: FnMut(&mut State, Prev::ItemOut) -> Out,
    State: Any,
{
    type Output<Next> = Prev::Output<MapScanPushSurfaceReversed<Next, Func, State, Prev::ItemOut>>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>;

    fn push_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    {
        self.prev
            .push_to(MapScanPushSurfaceReversed::new(next, self.func, self.state))
    }
}

pub struct MapScanPushSurfaceReversed<Next, Func, State, In>
where
    Next: PushSurfaceReversed,
    Func: FnMut(&mut State, In) -> Next::ItemIn,
    State: Any,
{
    next: Next,
    func: Func,
    state: State,
    _phantom: PhantomData<fn(In)>,
}
impl<Next, Func, State, In> MapScanPushSurfaceReversed<Next, Func, State, In>
where
    Next: PushSurfaceReversed,
    Func: FnMut(&mut State, In) -> Next::ItemIn,
    State: Any,
{
    pub fn new(next: Next, func: Func, state: State) -> Self {
        Self {
            next,
            func,
            state,
            _phantom: PhantomData,
        }
    }
}
impl<Next, Func, State, In> AssembleFlowGraph for MapScanPushSurfaceReversed<Next, Func, State, In>
where
    Next: PushSurfaceReversed + AssembleFlowGraph,
    Func: FnMut(&mut State, In) -> Next::ItemIn,
    State: Any,
{
    fn insert_dep(&self, e: &mut super::FlowGraph) -> FlowNodeId {
        let my_id = e.add_node("MapScan");
        let next_id = self.next.insert_dep(e);
        e.add_edge((my_id, next_id));
        my_id
    }
}

impl<Next, Func, State, In> PushSurfaceReversed
    for MapScanPushSurfaceReversed<Next, Func, State, In>
where
    Next: PushSurfaceReversed,
    Func: FnMut(&mut State, In) -> Next::ItemIn,
    State: Any,
{
    type ItemIn = In;

    type OutputHandoffs = Next::OutputHandoffs;
    type Build = MapPushBuild<Next::Build, MapScanFunc<Func, State, In, Next::ItemIn>, In>;

    fn make_parts(self, ctx: &mut Context) -> (Self::OutputHandoffs, Self::Build) {
        let state_handle = ctx.add_state(RefCell::new(self.state));

        let (connect, build) = self.next.make_parts(ctx);
        let build = MapPushBuild::new(build, wrap_func(self.func, state_handle));
        (connect, build)
    }
}
