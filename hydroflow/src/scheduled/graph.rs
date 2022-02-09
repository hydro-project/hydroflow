use std::any::Any;
use std::cell::Cell;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::mpsc::{self, Receiver, RecvError, SyncSender};

use ref_cast::RefCast;

use super::context::Context;
use super::handoff::handoff_list::PortList;
use super::handoff::{Handoff, HandoffMeta};
use super::port::{RecvCtx, RecvPort, SendCtx, SendPort, RECV, SEND};
use super::reactor::Reactor;
use super::state::StateHandle;
#[cfg(feature = "variadic_generics")]
use super::subgraph::Subgraph;
use super::{HandoffId, StateId, SubgraphId};

/// A Hydroflow graph. Owns, schedules, and runs the compiled subgraphs.
pub struct Hydroflow {
    subgraphs: Vec<SubgraphData>,
    handoffs: Vec<HandoffData>,

    states: Vec<StateData>,

    // TODO(mingwei): separate scheduler into its own struct/trait?
    ready_queue: VecDeque<SubgraphId>,
    event_queue_send: SyncSender<SubgraphId>, // TODO(mingwei) remove this, to prevent hanging.
    event_queue_recv: Receiver<SubgraphId>,
}
impl Default for Hydroflow {
    fn default() -> Self {
        let (subgraphs, handoffs, states, ready_queue) = Default::default();
        let (event_queue_send, event_queue_recv) = mpsc::sync_channel(8_000);
        Self {
            subgraphs,
            handoffs,
            states,
            ready_queue,
            event_queue_send,
            event_queue_recv,
        }
    }
}
impl Hydroflow {
    /// Create a new empty Hydroflow graph.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns a reactor for externally scheduling subgraphs, possibly from another thread.
    pub fn reactor(&self) -> Reactor {
        Reactor::new(self.event_queue_send.clone())
    }

    /// Runs the dataflow until no more work is currently available.
    pub fn tick(&mut self) {
        // Add any external jobs to ready queue.
        self.try_recv_events();

        while let Some(sg_id) = self.ready_queue.pop_front() {
            {
                let sg_data = &mut self.subgraphs[sg_id];
                // This must be true for the subgraph to be enqueued.
                assert!(sg_data.is_scheduled.take());

                let context = Context {
                    subgraph_id: sg_id,
                    handoffs: &mut self.handoffs,
                    states: &mut self.states,
                    event_queue_send: &mut self.event_queue_send,
                };
                sg_data.subgraph.run(context);
            }

            for &handoff_id in self.subgraphs[sg_id].succs.iter() {
                let handoff = &self.handoffs[handoff_id];
                if !handoff.handoff.is_bottom() {
                    for &succ_id in handoff.succs.iter() {
                        let succ_sg_data = &self.subgraphs[succ_id];
                        if succ_sg_data.is_scheduled.get() {
                            // Skip if task is already scheduled.
                            continue;
                        }
                        succ_sg_data.is_scheduled.set(true);
                        self.ready_queue.push_back(succ_id);
                    }
                }
            }

            self.try_recv_events();
        }
    }

    /// Run the dataflow graph to completion.
    ///
    /// TODO(mingwei): Currently blockes forever, no notion of "completion."
    pub fn run(&mut self) -> Result<!, RecvError> {
        loop {
            self.tick();
            self.recv_events()?;
        }
    }

    /// Run the dataflow graph to completion asynchronously
    ///
    /// TODO(mingwei): Currently blockes forever, no notion of "completion."
    pub async fn run_async(&mut self) -> Result<!, RecvError> {
        loop {
            self.tick();
            // Repeat until an external event triggers more subgraphs.
            while 0 == self.try_recv_events() {
                // TODO(mingwei): this busy-spins when other tasks are not running.
                tokio::task::yield_now().await;
            }
        }
    }

    /// Enqueues subgraphs triggered by external events without blocking.
    ///
    /// Returns the number of subgraphs enqueued.
    pub fn try_recv_events(&mut self) -> usize {
        let mut enqueued_count = 0;
        self.ready_queue.extend(
            self.event_queue_recv
                .try_iter()
                .filter(|&sg_id| !self.subgraphs[sg_id].is_scheduled.replace(true))
                .inspect(|_| enqueued_count += 1),
        );
        enqueued_count
    }

    /// Enqueues subgraphs triggered by external events, blocking until at
    /// least one subgraph is scheduled.
    pub fn recv_events(&mut self) -> Result<(), RecvError> {
        loop {
            let sg_id = self.event_queue_recv.recv()?;
            if !self.subgraphs[sg_id].is_scheduled.replace(true) {
                self.ready_queue.push_back(sg_id);

                // Enqueue any other immediate events.
                self.try_recv_events();

                return Ok(());
            }
        }
    }

    /// Adds a new compiled subgraph with the specified inputs and outputs.
    ///
    /// TODO(mingwei): add example in doc.
    pub fn add_subgraph<R, W, F>(
        &mut self,
        recv_ports: R,
        send_ports: W,
        mut subgraph: F,
    ) -> SubgraphId
    where
        R: 'static + PortList<RECV>,
        W: 'static + PortList<SEND>,
        F: 'static + FnMut(&Context<'_>, R::Ctx<'_>, W::Ctx<'_>),
    {
        let sg_id = self.subgraphs.len();

        let (mut subgraph_preds, mut subgraph_succs) = Default::default();
        recv_ports.set_graph_meta(&mut *self.handoffs, None, Some(sg_id), &mut subgraph_preds);
        send_ports.set_graph_meta(&mut *self.handoffs, Some(sg_id), None, &mut subgraph_succs);

        let subgraph = move |context: Context<'_>| {
            let recv = recv_ports.make_ctx(context.handoffs);
            let send = send_ports.make_ctx(context.handoffs);
            (subgraph)(&context, recv, send);
        };
        self.subgraphs.push(SubgraphData::new(
            subgraph,
            subgraph_preds,
            subgraph_succs,
            true,
        ));
        self.ready_queue.push_back(sg_id);

        sg_id
    }

    /// Adds a new compiled subraph with a variable number of inputs and outputs of the same respective handoff types.
    pub fn add_subgraph_n_m<R, W, F>(
        &mut self,
        recv_ports: Vec<RecvPort<R>>,
        send_ports: Vec<SendPort<W>>,
        mut subgraph: F,
    ) -> SubgraphId
    where
        R: 'static + Handoff,
        W: 'static + Handoff,
        F: 'static + FnMut(&Context<'_>, &[&RecvCtx<R>], &[&SendCtx<W>]),
    {
        let sg_id = self.subgraphs.len();

        let subgraph_preds = recv_ports.iter().map(|port| port.handoff_id).collect();
        let subgraph_succs = send_ports.iter().map(|port| port.handoff_id).collect();

        for recv_port in recv_ports.iter() {
            self.handoffs[recv_port.handoff_id].succs.push(sg_id);
        }
        for send_port in send_ports.iter() {
            self.handoffs[send_port.handoff_id].preds.push(sg_id);
        }

        let subgraph = move |context: Context<'_>| {
            let recvs: Vec<&RecvCtx<R>> = recv_ports
                .iter()
                .map(|hid| hid.handoff_id)
                .map(|hid| context.handoffs.get(hid).unwrap())
                .map(|h_data| {
                    h_data
                        .handoff
                        .any_ref()
                        .downcast_ref()
                        .expect("Attempted to cast handoff to wrong type.")
                })
                .map(RefCast::ref_cast)
                .collect();

            let sends: Vec<&SendCtx<W>> = send_ports
                .iter()
                .map(|hid| hid.handoff_id)
                .map(|hid| context.handoffs.get(hid).unwrap())
                .map(|h_data| {
                    h_data
                        .handoff
                        .any_ref()
                        .downcast_ref()
                        .expect("Attempted to cast handoff to wrong type.")
                })
                .map(RefCast::ref_cast)
                .collect();

            (subgraph)(&context, &recvs, &sends)
        };
        self.subgraphs.push(SubgraphData::new(
            subgraph,
            subgraph_preds,
            subgraph_succs,
            true,
        ));
        self.ready_queue.push_back(sg_id);

        sg_id
    }

    /// Creates a handoff edge and returns the corresponding send and receive ports.
    pub fn make_edge<H>(&mut self) -> (SendPort<H>, RecvPort<H>)
    where
        H: 'static + Handoff,
    {
        let handoff_id: HandoffId = self.handoffs.len();

        // Create and insert handoff.
        let handoff = H::default();
        self.handoffs.push(HandoffData::new(handoff));

        // Make ports.
        let input_port = SendPort {
            handoff_id,
            _marker: PhantomData,
        };
        let output_port = RecvPort {
            handoff_id,
            _marker: PhantomData,
        };
        (input_port, output_port)
    }

    pub fn add_state<T>(&mut self, state: T) -> StateHandle<T>
    where
        T: Any,
    {
        let state_id: StateId = self.states.len();

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

/// A handoff and its input and output [SubgraphId]s.
///
/// Internal use: used to track the hydroflow graph structure.
///
/// TODO(mingwei): restructure `PortList` so this can be crate-private.
pub struct HandoffData {
    pub(crate) handoff: Box<dyn HandoffMeta>,
    pub(crate) preds: Vec<SubgraphId>,
    pub(crate) succs: Vec<SubgraphId>,
}
impl std::fmt::Debug for HandoffData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("HandoffData")
            .field("preds", &self.preds)
            .field("succs", &self.succs)
            .finish_non_exhaustive()
    }
}
impl HandoffData {
    pub fn new(handoff: impl 'static + HandoffMeta) -> Self {
        let (preds, succs) = Default::default();
        Self {
            handoff: Box::new(handoff),
            preds,
            succs,
        }
    }
}

/// A subgraph along with its predecessor and successor [SubgraphId]s.
///
/// Used internally by the [Hydroflow] struct to represent the dataflow graph
/// structure and scheduled state.
struct SubgraphData {
    subgraph: Box<dyn Subgraph>,
    #[allow(dead_code)]
    preds: Vec<HandoffId>,
    succs: Vec<HandoffId>,
    /// If this subgraph is scheduled in [`Hydroflow::ready_queue`].
    /// [`Cell`] allows modifying this field when iterating `Self::preds` or
    /// `Self::succs`, as all `SubgraphData` are owned by the same vec
    /// `Hydroflow::subgraphs`.
    is_scheduled: Cell<bool>,
}
impl SubgraphData {
    pub fn new(
        subgraph: impl 'static + Subgraph,
        preds: Vec<HandoffId>,
        succs: Vec<HandoffId>,
        is_scheduled: bool,
    ) -> Self {
        Self {
            subgraph: Box::new(subgraph),
            preds,
            succs,
            is_scheduled: Cell::new(is_scheduled),
        }
    }
}

/// Internal struct containing a pointer to [`Hydroflow`]-owned state.
pub(crate) struct StateData {
    pub state: Box<dyn Any>,
}
