//! Module for [`StateHandle`], part of the "state API".

use std::marker::PhantomData;

use super::StateId;

/// A handle into a particular [`Hydroflow`](super::graph::Hydroflow) instance, referring to data
/// inserted by [`add_state`](super::graph::Hydroflow::add_state).
#[must_use]
#[derive(Debug)]
pub struct StateHandle<T> {
    pub(crate) state_id: StateId,
    pub(crate) _phantom: PhantomData<*mut T>,
}
impl<T> Copy for StateHandle<T> {}
impl<T> Clone for StateHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> StateHandle<T> {
    /// Returns the [`StateId`] of this handle.
    pub fn state_id(&self) -> StateId {
        self.state_id
    }
}
