use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use super::StateId;

#[must_use]
pub struct StatePort<T> {
    pub(crate) state_id: Rc<Cell<Option<StateId>>>,
    pub(crate) _phantom: PhantomData<fn() -> T>,
}

#[must_use]
pub struct StateHandle<T> {
    pub(crate) state_id: StateId,
    pub(crate) _phantom: PhantomData<*mut T>,
}
