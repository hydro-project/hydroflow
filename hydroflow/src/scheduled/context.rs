use std::any::Any;

use tokio::sync::mpsc::UnboundedSender;

use super::{
    graph::{HandoffData, StateData},
    state::StateHandle,
    SubgraphId,
};

// A handle onto the dataflow from within an individual operator.

pub struct Context<'a> {
    pub(crate) subgraph_id: SubgraphId,
    pub(crate) handoffs: &'a mut [HandoffData],
    pub(crate) states: &'a mut [StateData],
    pub(crate) event_queue_send: &'a UnboundedSender<SubgraphId>,
    pub(crate) current_epoch: usize,
    pub(crate) current_stratum: usize,
}
impl<'a> Context<'a> {
    // Gets the current epoch (local time) count.
    pub fn current_epoch(&self) -> usize {
        self.current_epoch
    }

    // Gets the current stratum nubmer.
    pub fn current_stratum(&self) -> usize {
        self.current_stratum
    }

    pub fn waker(&self) -> std::task::Waker {
        use futures::task::ArcWake;
        use std::sync::Arc;

        struct ContextWaker {
            subgraph_id: SubgraphId,
            event_queue_send: UnboundedSender<SubgraphId>,
        }
        impl ArcWake for ContextWaker {
            fn wake_by_ref(arc_self: &Arc<Self>) {
                arc_self.event_queue_send.send(arc_self.subgraph_id).unwrap(/* TODO(mingwei) */);
            }
        }

        let context_waker = ContextWaker {
            subgraph_id: self.subgraph_id,
            event_queue_send: self.event_queue_send.clone(),
        };
        futures::task::waker(Arc::new(context_waker))
    }

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
}
