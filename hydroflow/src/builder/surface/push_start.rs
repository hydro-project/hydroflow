use super::{BaseSurface, PushSurface, PushSurfaceReversed, StoreDataflowGraph};

use std::marker::PhantomData;

pub struct StartPushSurface<Out> {
    _phantom: PhantomData<fn(Out)>,
}

impl<Out> Default for StartPushSurface<Out> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<Out> StartPushSurface<Out> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<Out> StoreDataflowGraph for StartPushSurface<Out> {
    fn insert_dep(&self, e: &mut super::DataflowGraphStorage) -> usize {
        let my_id = e.add_node("Start".to_string());
        my_id
    }
}

impl<Out> BaseSurface for StartPushSurface<Out> {
    type ItemOut = Out;
}

impl<Out> PushSurface for StartPushSurface<Out> {
    type Output<Next> = Next
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>;

    fn push_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: PushSurfaceReversed<ItemIn = Self::ItemOut>,
    {
        next
    }
}
