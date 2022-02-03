use std::any::Any;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::mpsc::{self, Receiver, RecvError, SyncSender};

use ref_cast::RefCast;

use super::context::Context;
use super::handoff::{Handoff, HandoffMeta, RecvPortList, SendPortList};
use super::port::{InputPort, OutputPort, RecvCtx, SendCtx};
use super::reactor::Reactor;
use super::state::StateHandle;
#[cfg(feature = "variadic_generics")]
use super::subgraph::Subgraph;
use super::{HandoffId, StateId, SubgraphId};

/**
 * A Hydroflow graph. Owns, schedules, and runs the compiled subgraphs.
 */
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
    /**
     * Create an new empty Dataflow graph.
     */
    pub fn new() -> Self {
        Default::default()
    }

    /**
     * Returns a reactor for externally scheduling subgraphs, possibly from another thread.
     */
    pub fn reactor(&self) -> Reactor {
        Reactor::new(self.event_queue_send.clone())
    }

    fn enqueue_jobs(&mut self) {
        for sg in self.event_queue_recv.try_iter() {
            if !self.subgraphs[sg].scheduled {
                self.ready_queue.push_back(sg);
                self.subgraphs[sg].scheduled = true;
            }
        }
    }

    /**
     * Runs the dataflow until no more work is currently available.
     */
    pub fn tick(&mut self) {
        // Add any external jobs to ready queue.
        self.enqueue_jobs();

        while let Some(sg_id) = self.ready_queue.pop_front() {
            self.subgraphs[sg_id].scheduled = false;
            let sg_data = self.subgraphs.get_mut(sg_id).unwrap(/* TODO(mingwei) */);
            let context = Context {
                subgraph_id: sg_id,
                handoffs: &mut self.handoffs,
                states: &mut self.states,
                event_queue_send: &mut self.event_queue_send,
            };
            sg_data.subgraph.run(context);
            for &handoff_id in sg_data.succs.iter() {
                let handoff = self.handoffs.get(handoff_id).unwrap(/* TODO(mingwei) */);
                for &succ_id in handoff.succs.iter() {
                    if self.ready_queue.contains(&succ_id) {
                        // TODO(mingwei): Slow? O(N)
                        continue;
                    }
                    if !handoff.handoff.is_bottom() {
                        self.ready_queue.push_back(succ_id);
                    }
                }
            }

            self.enqueue_jobs();
        }
    }

    /**
     * Run the dataflow graph, blocking until completion.
     */
    pub fn run(&mut self) -> Result<!, RecvError> {
        loop {
            self.tick();
            self.poll_events()?;
        }
    }

    /**
     * Run the dataflow graph to completion asynchronously.
     */
    pub async fn run_async(&mut self) -> Result<!, RecvError> {
        loop {
            self.tick();
            self.poll_events()?;
            tokio::task::yield_now().await;
            // TODO(mingwei): this busy-spins when other tasks are not running.
        }
    }

    /**
     * Block and wait for an external event.
     */
    pub fn poll_events(&mut self) -> Result<(), RecvError> {
        self.ready_queue.extend(self.event_queue_recv.try_iter());
        Ok(())
    }

    /**
     * TODO(mingwei): Hack to re-enqueue all subgraphs.
     */
    pub fn wake_all(&mut self) {
        self.ready_queue.clear();
        self.ready_queue.extend(0..self.subgraphs.len());
    }

    /// Adds a new compiled subgraph with the specified inputs and outputs.
    ///
    /// See [TODO] for how to specify inputs and outputs.
    pub fn add_subgraph<R, W, F>(
        &mut self,
        recv_ports: R,
        send_ports: W,
        mut subgraph: F,
    ) -> SubgraphId
    where
        R: 'static + RecvPortList,
        W: 'static + SendPortList,
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
        self.subgraphs
            .push(SubgraphData::new(subgraph, subgraph_preds, subgraph_succs));
        self.ready_queue.push_back(sg_id);

        sg_id
    }

    /// Adds a new compiled subraph with a variable number of inputs and outputs of the same respective handoff types.
    pub fn add_subgraph_n_m<R, W, F>(
        &mut self,
        recv_ports: Vec<OutputPort<R>>,
        send_ports: Vec<InputPort<W>>,
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
        self.subgraphs
            .push(SubgraphData::new(subgraph, subgraph_preds, subgraph_succs));
        self.ready_queue.push_back(sg_id);

        sg_id
    }

    pub fn make_handoff<H>(&mut self) -> (InputPort<H>, OutputPort<H>)
    where
        H: 'static + Handoff,
    {
        let handoff_id: HandoffId = self.handoffs.len();

        // Create and insert handoff.
        let handoff = H::default();
        self.handoffs.push(HandoffData::new(handoff));

        // Make ports.
        let input_port = InputPort {
            handoff_id,
            _marker: PhantomData,
        };
        let output_port = OutputPort {
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

/**
 * A handoff and its input and output [SubgraphId]s.
 *
 * NOT PART OF PUBLIC API.
 */
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

/**
 * A subgraph along with its predecessor and successor [SubgraphId]s.
 * Used internally by the [Hydroflow] struct to represent the dataflow graph structure.
 */
struct SubgraphData {
    subgraph: Box<dyn Subgraph>,
    #[allow(dead_code)]
    preds: Vec<HandoffId>,
    succs: Vec<HandoffId>,
    scheduled: bool,
}
impl SubgraphData {
    pub fn new(
        subgraph: impl 'static + Subgraph,
        preds: Vec<HandoffId>,
        succs: Vec<HandoffId>,
    ) -> Self {
        Self {
            subgraph: Box::new(subgraph),
            preds,
            succs,
            scheduled: true,
        }
    }
}

pub struct StateData {
    pub state: Box<dyn Any>,
}
