//! Module for the user-facing [`Context`] object.
#![deny(missing_docs)]

use std::any::Any;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use super::graph::StateData;
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
    pub(crate) states: Vec<StateData>,

    // TODO(mingwei): as long as this is here, it's impossible to know when all work is done.
    // Second field (bool) is for if the event is an external "important" event (true).
    pub(crate) event_queue_send: UnboundedSender<(SubgraphId, bool)>,

    pub(crate) current_tick: usize,
    pub(crate) current_stratum: usize,

    /// The SubgraphId of the currently running operator. When this context is
    /// not being forwarded to a running operator, this field is (mostly)
    /// meaningless.
    pub(crate) subgraph_id: SubgraphId,

    /// Join handles for spawned tasks.
    pub(crate) task_join_handles: Vec<JoinHandle<()>>,

    /// Number of backpressure events triggered. Backpressure is binary: if anything triggers
    /// backpressure the graph will block until all backpressure triggers are released.
    pub(crate) backpresure_count: Arc<BackpressureInner>,
}
impl Context {
    pub(crate) fn new(event_queue_send: UnboundedSender<(SubgraphId, bool)>) -> Self {
        Self {
            states: Vec::new(),

            event_queue_send,

            current_stratum: 0,
            current_tick: 0,

            subgraph_id: SubgraphId(0),

            task_join_handles: Vec::new(),

            backpresure_count: Default::default(),
        }
    }

    /// Gets the current tick (local time) count.
    pub fn current_tick(&self) -> usize {
        self.current_tick
    }

    /// Gets the current stratum nubmer.
    pub fn current_stratum(&self) -> usize {
        self.current_stratum
    }

    /// Gets the ID of the current subgraph.
    pub fn current_subgraph(&self) -> SubgraphId {
        self.subgraph_id
    }

    /// Schedules a subgraph.
    pub fn schedule_subgraph(&self, sg_id: SubgraphId, is_external: bool) {
        self.event_queue_send.send((sg_id, is_external)).unwrap()
    }

    /// Returns a `Waker` for interacting with async Rust.
    /// Waker events are considered to be extenral.
    pub fn waker(&self) -> std::task::Waker {
        use futures::task::ArcWake;

        struct ContextWaker {
            subgraph_id: SubgraphId,
            event_queue_send: UnboundedSender<(SubgraphId, bool)>,
        }
        impl ArcWake for ContextWaker {
            fn wake_by_ref(arc_self: &Arc<Self>) {
                let _recv_closed_error =
                    arc_self.event_queue_send.send((arc_self.subgraph_id, true));
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

    /// Removes state from the context returns it as an owned heap value.
    pub fn remove_state<T>(&mut self, handle: StateHandle<T>) -> Box<T>
    where
        T: Any,
    {
        self.states
            .remove(handle.state_id.0)
            .state
            .downcast()
            .expect("StateHandle wrong type T for casting.")
    }

    /// Spawns an async task on the internal Tokio executor.
    pub fn spawn_task<Fut>(&mut self, future: Fut)
    where
        Fut: Future<Output = ()> + 'static,
    {
        self.task_join_handles
            .push(tokio::task::spawn_local(future));
    }

    /// Aborts all tasks spawned with [`Self::spawn_task`].
    pub fn abort_tasks(&mut self) {
        for task in self.task_join_handles.drain(..) {
            task.abort();
        }
    }

    /// Waits for all tasks spawned with [`Self::spawn_task`] to complete.
    ///
    /// Will probably just hang.
    pub async fn join_tasks(&mut self) {
        futures::future::join_all(self.task_join_handles.drain(..)).await;
    }

    /// Return a handle to a [`BackpressureValve`] for triggering (and later releasing) backpressure.
    pub fn backpressure_valve(&self) -> BackpressureValve {
        BackpressureValve {
            backpresure_count: Arc::clone(&self.backpresure_count),
        }
    }
}

#[derive(Default)]
pub(crate) struct BackpressureInner {
    pub(crate) notify: Notify,
    pub(crate) count: AtomicUsize,
}

/// Represents control over backpressure for a Hydroflow instance.
#[derive(Clone)]
pub struct BackpressureValve {
    backpresure_count: Arc<BackpressureInner>,
}

impl BackpressureValve {
    /// Trigger backpressure. It is an error to call this if backpressure is currently triggered.
    pub fn trigger(&self) {
        self.backpresure_count.count.fetch_add(1, Ordering::Relaxed);
    }

    /// Release backpressure. It is an error to call this if backpressure is not currently triggered.
    pub fn release(&self) {
        if 0 == self.backpresure_count.count.fetch_sub(1, Ordering::Relaxed) {
            self.backpresure_count.notify.notify_one();
        }
    }
}
