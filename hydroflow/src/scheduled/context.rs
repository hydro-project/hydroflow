use std::{any::Any, marker::PhantomData};

use tokio::sync::mpsc::UnboundedSender;

use super::graph::{HandoffData, StateData};
use super::state::StateHandle;
use super::{StateId, SubgraphId};

/// The main state of the Hydroflow instance, which is provided as a reference
/// to each operator as it is run.
///
/// As an optimization, each Hydroflow instances stores eactly one Context
/// inline, which allows us to avoid any construction/deconstruction costs.
/// Before the `Context` is provided to a running operator, the `subgraph_id`
/// field must be updated.
pub struct Context {
    pub(crate) handoffs: Vec<HandoffData>,
    pub(crate) states: Vec<StateData>,

    // TODO(mingwei): as long as this is here, it's impossible to know when all work is done.
    pub(crate) event_queue_send: UnboundedSender<SubgraphId>,

    pub(crate) current_epoch: usize,
    pub(crate) current_stratum: usize,

    /// The SubgraphId of the currently running operator. When this context is
    /// not being forwarded to a running operator, this field is (mostly)
    /// meaningless.
    pub(crate) subgraph_id: SubgraphId,
}
impl Context {
    /// Gets the current epoch (local time) count.
    pub fn current_epoch(&self) -> usize {
        self.current_epoch
    }

    /// Gets the current stratum nubmer.
    pub fn current_stratum(&self) -> usize {
        self.current_stratum
    }

    /// Schedules a subgraph.
    pub fn schedule_subgraph(&self, sg_id: SubgraphId) {
        self.event_queue_send.send(sg_id).unwrap()
    }

    /// Returns a `Waker` for interacting with async Rust.
    pub fn waker(&self) -> std::task::Waker {
        use futures::task::ArcWake;
        use std::sync::Arc;

        struct ContextWaker {
            subgraph_id: SubgraphId,
            event_queue_send: UnboundedSender<SubgraphId>,
        }
        impl ArcWake for ContextWaker {
            fn wake_by_ref(arc_self: &Arc<Self>) {
                let _recv_closed_error = arc_self.event_queue_send.send(arc_self.subgraph_id);
            }
        }

        let context_waker = ContextWaker {
            subgraph_id: self.subgraph_id,
            event_queue_send: self.event_queue_send.clone(),
        };
        futures::task::waker(Arc::new(context_waker))
    }

    /// Returns a shared reference to the state.
    pub fn state_ref<T>(&self, handle: StateHandle<T>) -> &T
    where
        T: Any,
    {
        self.states
            .get(handle.state_id.0)
            .expect("Failed to find state with given handle.")
            .state
            .downcast_ref()
            .expect("StateHandle wrong type T for casting.")
    }

    /// Returns an exclusive reference to the state.
    pub fn state_mut<T>(&mut self, handle: StateHandle<T>) -> &mut T
    where
        T: Any,
    {
        self.states
            .get_mut(handle.state_id.0)
            .expect("Failed to find state with given handle.")
            .state
            .downcast_mut()
            .expect("StateHandle wrong type T for casting.")
    }

    /// Adds state to the context and returns the handle.
    pub fn add_state<T>(&mut self, state: T) -> StateHandle<T>
    where
        T: Any,
    {
        let state_id = StateId(self.states.len());

        let state_data = StateData {
            state: Box::new(state),
        };
        self.states.push(state_data);

        StateHandle {
            state_id,
            _phantom: PhantomData,
        }
    }
}
