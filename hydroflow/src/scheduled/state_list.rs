use std::any::Any;
use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::scheduled::state::StatePort;
use crate::scheduled::{StateData, StateId};

pub trait StateList {
    type StateId;
    type StatePort;
    fn make_port() -> (Self::StateId, Self::StatePort);

    type StateRef<'a>;
    fn make_refs<'a>(states: &'a [StateData], state_ids: &Self::StateId) -> Self::StateRef<'a>;
}
impl<T, L> StateList for (T, L)
where
    T: Any,
    L: StateList,
{
    type StateId = (Rc<Cell<Option<StateId>>>, L::StateId);
    type StatePort = (StatePort<T>, L::StatePort);
    fn make_port() -> (Self::StateId, Self::StatePort) {
        let sid = <Rc<Cell<Option<StateId>>>>::default();

        let port = StatePort {
            state_id: sid.clone(),
            _phantom: PhantomData,
        };

        let (sid_rest, port_rest) = L::make_port();

        ((sid, sid_rest), (port, port_rest))
    }

    type StateRef<'a> = (&'a T, L::StateRef<'a>);
    fn make_refs<'a>(states: &'a [StateData], state_ids: &Self::StateId) -> Self::StateRef<'a> {
        let (sid, sid_rest) = state_ids;

        let sid = sid.get().expect("Attempted to use unaattached state.");
        let state = states
            .get(sid)
            .unwrap()
            .state
            .downcast_ref()
            .expect("Attempted to cast handoff to wrong type.");

        let state_rest = L::make_refs(states, sid_rest);
        (state, state_rest)
    }
}
impl StateList for () {
    type StateId = ();
    type StatePort = ();
    fn make_port() -> (Self::StateId, Self::StatePort) {
        ((), ())
    }

    type StateRef<'a> = ();
    fn make_refs<'a>(_states: &'a [StateData], _state_ids: &Self::StateId) -> Self::StateRef<'a> {}
}
