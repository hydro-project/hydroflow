use std::any::Any;
use std::cell::Cell;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, RecvError, SyncSender};

use ref_cast::RefCast;

use super::context::Context;
use super::ctx::{InputPort, OutputPort, RecvCtx, SendCtx};
use super::handoff::{Handoff, HandoffMeta};
use super::handoff_list::HandoffList;
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
            for handoff_id in sg_data.succs.iter().copied() {
                let handoff = self.handoffs.get(handoff_id).unwrap(/* TODO(mingwei) */);
                let succ_id = handoff.succ;
                if self.ready_queue.contains(&succ_id) {
                    // TODO(mingwei): Slow? O(N)
                    continue;
                }
                if !handoff.handoff.is_bottom() {
                    self.ready_queue.push_back(succ_id);
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

    /**
     * Adds a new compiled subgraph with the specified inputs and outputs.
     *
     * See [HandoffList] for how to specify inputs and outputs.
     */
    #[cfg(feature = "variadic_generics")]
    #[must_use]
    pub fn add_subgraph<F, R, W>(&mut self, mut subgraph: F) -> (R::InputPort, W::OutputPort)
    where
        F: 'static + FnMut(&Context<'_>, R::RecvCtx<'_>, W::SendCtx<'_>),
        R: 'static + HandoffList,
        W: 'static + HandoffList,
    {
        // TODO(justin): make this less sketchy, we just know we're the only person who will append here.
        let sg_id = self.subgraphs.len();

        let (input_hids, input_ports) = R::make_input(sg_id);
        let (output_hids, output_ports) = W::make_output(sg_id);

        let subgraph = move |context: Context<'_>| {
            let recv = R::make_recv(context.handoffs, &input_hids);
            let send = W::make_send(context.handoffs, &output_hids);
            (subgraph)(&context, recv, send);
        };
        self.subgraphs.push(SubgraphData::new(subgraph));
        self.ready_queue.push_back(sg_id);

        (input_ports, output_ports)
    }

    /**
     * Adds a new compiled subraph with a variable number of inputs and outputs.
     */
    pub fn add_n_in_m_out<F, R, W>(
        &mut self,
        n: usize,
        m: usize,
        f: F,
    ) -> (Vec<InputPort<R>>, Vec<OutputPort<W>>)
    where
        F: 'static + FnMut(&[&RecvCtx<R>], &[&SendCtx<W>]),
        R: 'static + Handoff,
        W: 'static + Handoff,
    {
        // TODO(justin): is there a nice way to encapsulate the below?
        let sg_id = self.subgraphs.len();

        let mut input_hids = Vec::new();
        input_hids.resize_with(n, <Rc<Cell<Option<HandoffId>>>>::default);
        let mut output_hids = Vec::new();
        output_hids.resize_with(m, <Rc<Cell<Option<HandoffId>>>>::default);

        let input_ports = input_hids
            .iter()
            .cloned()
            .map(|handoff_id| InputPort {
                sg_id,
                handoff_id,
                _phantom: PhantomData,
            })
            .collect();
        let output_ports = output_hids
            .iter()
            .cloned()
            .map(|handoff_id| OutputPort {
                sg_id,
                handoff_id,
                _phantom: PhantomData,
            })
            .collect();

        let mut f = f;
        let subgraph = move |context: Context<'_>| {
            let recvs: Vec<&RecvCtx<R>> = input_hids
                .iter()
                .map(|hid| hid.get().expect("Attempted to use unattached handoff."))
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

            let sends: Vec<&SendCtx<W>> = output_hids
                .iter()
                .map(|hid| hid.get().expect("Attempted to use unattached handoff."))
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

            // self.handoffs.
            f(&recvs, &sends)
        };
        self.subgraphs.push(SubgraphData::new(subgraph));
        self.ready_queue.push_back(sg_id);

        (input_ports, output_ports)
    }

    pub fn add_edge<H>(&mut self, output_port: OutputPort<H>, input_port: InputPort<H>)
    where
        H: 'static + Handoff,
    {
        let handoff_id = self.add_handoff::<H>(output_port.sg_id, input_port.sg_id);

        // Send handoff_ids.
        input_port.handoff_id.set(Some(handoff_id));
        output_port.handoff_id.set(Some(handoff_id));
    }

    pub fn add_handoff<H: Handoff>(
        &mut self,
        pred_id: SubgraphId,
        succ_id: SubgraphId,
    ) -> HandoffId {
        let handoff_id: HandoffId = self.handoffs.len();

        // Create and insert handoff.
        let handoff = H::default();
        self.handoffs
            .push(HandoffData::new(handoff, pred_id, succ_id));

        // Add successor & predecessor.
        self.subgraphs[pred_id].succs.push(handoff_id);
        self.subgraphs[succ_id].preds.push(handoff_id);

        handoff_id
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
    pub handoff: Box<dyn HandoffMeta>,
    #[allow(dead_code)] // TODO(mingwei)
    pred: SubgraphId,
    succ: SubgraphId,
}
impl HandoffData {
    pub fn new(handoff: impl 'static + HandoffMeta, pred: SubgraphId, succ: SubgraphId) -> Self {
        Self {
            handoff: Box::new(handoff),
            pred,
            succ,
        }
    }
}

/**
 * A subgraph along with its predecessor and successor [SubgraphId]s.
 * Used internally by the [Hydroflow] struct to represent the dataflow graph structure.
 */
struct SubgraphData {
    subgraph: Box<dyn Subgraph>,
    preds: Vec<HandoffId>,
    succs: Vec<HandoffId>,
    scheduled: bool,
}
impl SubgraphData {
    pub fn new(subgraph: impl 'static + Subgraph) -> Self {
        Self {
            subgraph: Box::new(subgraph),
            preds: Default::default(),
            succs: Default::default(),
            scheduled: true,
        }
    }
}

pub struct StateData {
    pub state: Box<dyn Any>,
}
