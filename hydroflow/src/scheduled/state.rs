use std::marker::PhantomData;

use super::StateId;

#[must_use]
pub struct StateHandle<T> {
    pub(crate) state_id: StateId,
    pub(crate) _phantom: PhantomData<*mut T>,
}
impl<T> Clone for StateHandle<T> {
    fn clone(&self) -> Self {
        Self {
            state_id: self.state_id,
            _phantom: PhantomData,
        }
    }
}
impl<T> Copy for StateHandle<T> {}
