use std::any::Any;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::{HashSet, VecDeque};
use std::fmt::Write;
use std::marker::PhantomData;
use std::num::NonZeroUsize;

use ref_cast::RefCast;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use super::context::Context;
use super::handoff::handoff_list::PortList;
use super::handoff::{Handoff, HandoffMeta};
use super::port::{RecvCtx, RecvPort, SendCtx, SendPort, RECV, SEND};
use super::reactor::Reactor;
use super::state::StateHandle;
use super::subgraph::Subgraph;
use super::{HandoffId, StateId, SubgraphId};

/// A Hydroflow graph. Owns, schedules, and runs the compiled subgraphs.
pub struct Hydroflow {
    subgraphs: Vec<SubgraphData>,
    handoffs: Vec<HandoffData>,

    states: Vec<StateData>,

    // TODO(mingwei): separate scheduler into its own struct/trait?
    // Index is stratum, value is FIFO queue for that stratum.
    stratum_queues: Vec<VecDeque<SubgraphId>>,
    current_stratum: usize,
    current_epoch: usize,

    event_queue_send: UnboundedSender<SubgraphId>, // TODO(mingwei) remove this, to prevent hanging.
    event_queue_recv: UnboundedReceiver<SubgraphId>,
}
impl Default for Hydroflow {
    fn default() -> Self {
        let (subgraphs, handoffs, states, stratum_queues) = Default::default();
        let (event_queue_send, event_queue_recv) = mpsc::unbounded_channel();
        Self {
            subgraphs,
            handoffs,
            states,

            stratum_queues,
            current_stratum: 0,
            current_epoch: 0,

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

    // Gets the current epoch (local time) count.
    pub fn current_epoch(&self) -> usize {
        self.current_epoch
    }

    // Gets the current stratum nubmer.
    pub fn current_stratum(&self) -> usize {
        self.current_stratum
    }

    /// Runs the dataflow until no more work is immediately available.
    pub fn tick(&mut self) {
        while self.next_stratum() {
            self.tick_stratum();
        }
    }

    /// Runs the current stratum of the dataflow until no more work is immediately available.
    pub fn tick_stratum(&mut self) {
        // Add any external jobs to ready queue.
        self.try_recv_events();

        while let Some(sg_id) = self.stratum_queues[self.current_stratum].pop_front() {
            {
                let sg_data = &mut self.subgraphs[sg_id];
                // This must be true for the subgraph to be enqueued.
                assert!(sg_data.is_scheduled.take());

                let context = Context {
                    subgraph_id: sg_id,
                    handoffs: &mut self.handoffs,
                    states: &mut self.states,
                    event_queue_send: &self.event_queue_send,
                    current_epoch: self.current_epoch,
                    current_stratum: self.current_stratum,
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
                        self.stratum_queues[succ_sg_data.stratum].push_back(succ_id);
                    }
                }
            }

            self.try_recv_events();
        }
    }

    /// Go to the next stratum which has work available, possibly the current stratum.
    /// Return true if more work is available, otherwise false if no work is immediately available on any strata.
    pub fn next_stratum(&mut self) -> bool {
        self.try_recv_events();

        let old_stratum = self.current_stratum;
        loop {
            // If current stratum has work, return true.
            if !self.stratum_queues[self.current_stratum].is_empty() {
                return true;
            }
            // Increment stratum counter.
            self.current_stratum += 1;
            if self.current_stratum >= self.stratum_queues.len() {
                self.current_stratum = 0;
                self.current_epoch += 1;
            }
            // After incrementing, exit if we made a full loop around the strata.
            if old_stratum == self.current_stratum {
                // Note: if current stratum had work, the very first loop iteration would've
                // returned true. Therefore we can return false without checking.
                return false;
            }
        }
    }

    /// Run the dataflow graph to completion.
    ///
    /// TODO(mingwei): Currently blockes forever, no notion of "completion."
    pub fn run(&mut self) -> Option<!> {
        loop {
            self.tick();
            self.recv_events()?;
        }
    }

    /// Run the dataflow graph to completion asynchronously
    ///
    /// TODO(mingwei): Currently blockes forever, no notion of "completion."
    pub async fn run_async(&mut self) -> Option<!> {
        loop {
            self.tick();
            self.recv_events_async().await?;
            tokio::task::yield_now().await;
        }
    }

    /// Enqueues subgraphs triggered by external events without blocking.
    ///
    /// Returns the number of subgraphs enqueued.
    pub fn try_recv_events(&mut self) -> usize {
        let mut enqueued_count = 0;
        while let Ok(sg_id) = self.event_queue_recv.try_recv() {
            let sg_data = &self.subgraphs[sg_id];
            if !sg_data.is_scheduled.replace(true) {
                self.stratum_queues[sg_data.stratum].push_back(sg_id);
                enqueued_count += 1;
            }
        }
        enqueued_count
    }

    /// Enqueues subgraphs triggered by external events, blocking until at
    /// least one subgraph is scheduled.
    pub fn recv_events(&mut self) -> Option<NonZeroUsize> {
        loop {
            let sg_id = self.event_queue_recv.blocking_recv()?;
            let sg_data = &self.subgraphs[sg_id];
            if !sg_data.is_scheduled.replace(true) {
                self.stratum_queues[sg_data.stratum].push_back(sg_id);

                // Enqueue any other immediate events.
                return Some(NonZeroUsize::new(self.try_recv_events() + 1).unwrap());
            }
        }
    }

    /// Enqueues subgraphs triggered by external events asynchronously, waiting
    /// until at least one subgraph is scheduled.
    pub async fn recv_events_async(&mut self) -> Option<NonZeroUsize> {
        loop {
            let sg_id = self.event_queue_recv.recv().await?;
            let sg_data = &self.subgraphs[sg_id];
            if !sg_data.is_scheduled.replace(true) {
                self.stratum_queues[sg_data.stratum].push_back(sg_id);

                // Enqueue any other immediate events.
                return Some(NonZeroUsize::new(self.try_recv_events() + 1).unwrap());
            }
        }
    }

    pub fn add_subgraph<Name, R, W, F>(
        &mut self,
        name: Name,
        recv_ports: R,
        send_ports: W,
        subgraph: F,
    ) -> SubgraphId
    where
        Name: Into<Cow<'static, str>>,
        R: 'static + PortList<RECV>,
        W: 'static + PortList<SEND>,
        F: 'static + for<'ctx> FnMut(&'ctx Context<'ctx>, R::Ctx<'ctx>, W::Ctx<'ctx>),
    {
        self.add_subgraph_stratified(name, 0, recv_ports, send_ports, subgraph)
    }

    /// Adds a new compiled subgraph with the specified inputs and outputs.
    ///
    /// TODO(mingwei): add example in doc.
    pub fn add_subgraph_stratified<Name, R, W, F>(
        &mut self,
        name: Name,
        stratum: usize,
        recv_ports: R,
        send_ports: W,
        mut subgraph: F,
    ) -> SubgraphId
    where
        Name: Into<Cow<'static, str>>,
        R: 'static + PortList<RECV>,
        W: 'static + PortList<SEND>,
        F: 'static + for<'ctx> FnMut(&'ctx Context<'ctx>, R::Ctx<'ctx>, W::Ctx<'ctx>),
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
            name.into(),
            stratum,
            subgraph,
            subgraph_preds,
            subgraph_succs,
            DirectedEdgeSet::default(),
            true,
        ));
        self.init_stratum(stratum);
        self.stratum_queues[stratum].push_back(sg_id);

        sg_id
    }

    /// Adds a new compiled subgraph with a variable number of inputs and outputs of the same respective handoff types.
    pub fn add_subgraph_n_m<Name, R, W, F>(
        &mut self,
        name: Name,
        recv_ports: Vec<RecvPort<R>>,
        send_ports: Vec<SendPort<W>>,
        subgraph: F,
    ) -> SubgraphId
    where
        Name: Into<Cow<'static, str>>,
        R: 'static + Handoff,
        W: 'static + Handoff,
        F: 'static
            + for<'ctx> FnMut(&'ctx Context<'ctx>, &'ctx [&'ctx RecvCtx<R>], &'ctx [&'ctx SendCtx<W>]),
    {
        self.add_subgraph_stratified_n_m(name, 0, recv_ports, send_ports, subgraph)
    }

    /// Adds a new compiled subgraph with a variable number of inputs and outputs of the same respective handoff types.
    pub fn add_subgraph_stratified_n_m<Name, R, W, F>(
        &mut self,
        name: Name,
        stratum: usize,
        recv_ports: Vec<RecvPort<R>>,
        send_ports: Vec<SendPort<W>>,
        mut subgraph: F,
    ) -> SubgraphId
    where
        Name: Into<Cow<'static, str>>,
        R: 'static + Handoff,
        W: 'static + Handoff,
        F: 'static
            + for<'ctx> FnMut(&'ctx Context<'ctx>, &'ctx [&'ctx RecvCtx<R>], &'ctx [&'ctx SendCtx<W>]),
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
            name.into(),
            stratum,
            subgraph,
            subgraph_preds,
            subgraph_succs,
            DirectedEdgeSet::default(),
            true,
        ));
        self.init_stratum(stratum);
        self.stratum_queues[stratum].push_back(sg_id);

        sg_id
    }

    /// Makes sure stratum STRATUM is initialized.
    fn init_stratum(&mut self, stratum: usize) {
        if self.stratum_queues.len() <= stratum {
            self.stratum_queues
                .resize_with(stratum + 1, Default::default);
        }
    }

    /// Creates a handoff edge and returns the corresponding send and receive ports.
    pub fn make_edge<Name, H>(&mut self, name: Name) -> (SendPort<H>, RecvPort<H>)
    where
        Name: Into<Cow<'static, str>>,
        H: 'static + Handoff,
    {
        let handoff_id: HandoffId = self.handoffs.len();

        // Create and insert handoff.
        let handoff = H::default();
        self.handoffs.push(HandoffData::new(name.into(), handoff));

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

    pub fn add_dependencies(&mut self, sg_id: SubgraphId, deps: DirectedEdgeSet) {
        self.subgraphs[sg_id].dependencies.append(deps);
    }

    fn mermaid_mangle(&self, name: Cow<'static, str>, subg: usize, node: usize) -> String {
        let res = if name.starts_with("Handoff") {
            let handoff_id: usize = name.split('_').collect::<Vec<&str>>()[1].parse().unwrap();
            let handoff = &self.handoffs[handoff_id];
            // let name = self.remove_whitespace(handoff.name.clone());
            format!("Handoff:_{}[\\{}/]", handoff_id, handoff.name)
            // name
        } else if &*name == "PullToPush" {
            format!("{}.{}[/{}\\]", subg, node, name)
        } else {
            // format!("{}.{}({})", subg, node, name)
            format!("{}.{}[{}]", subg, node, name)
        };
        res
    }

    pub fn render_mermaid(&self) -> String {
        let mut retval = "graph TD\n".to_string();
        for i in 0..self.subgraphs.len() {
            let d = &self.subgraphs[i].dependencies;
            let name = &self.subgraphs[i].name;
            let stratum = self.subgraphs[i].stratum;

            if !d.edges.is_empty() {
                writeln!(retval, "subgraph stratum{}", stratum).unwrap();
                writeln!(retval, "subgraph {}{}", name, i).unwrap();
                for e in &d.edges {
                    let from = self.mermaid_mangle(d.node_names[e.0].clone(), i, e.0);
                    let to = self.mermaid_mangle(d.node_names[e.1].clone(), i, e.1);
                    writeln!(retval, "{} --> {}", from, to,).unwrap();
                }
                writeln!(retval, "end").unwrap();
                writeln!(retval, "end").unwrap();
            }
        }
        retval
    }
}

#[derive(Debug, Default)]
pub struct DirectedEdgeSet {
    pub node_names: Vec<Cow<'static, str>>,
    pub edges: HashSet<(usize, usize)>,
}
impl DirectedEdgeSet {
    pub fn add_node(&mut self, name: impl Into<Cow<'static, str>>) -> usize {
        let current_index = self.node_names.len();
        self.node_names.insert(current_index, name.into());
        current_index
    }
    pub fn add_edge(&mut self, edge: (usize, usize)) {
        self.edges.insert(edge);
    }
    pub fn append(&mut self, mut other: DirectedEdgeSet) {
        let base = self.node_names.len();
        self.node_names.append(&mut other.node_names);
        self.edges.extend(
            other
                .edges
                .into_iter()
                .map(|(from, to)| (from + base, to + base)),
        );
    }
}
/// A handoff and its input and output [SubgraphId]s.
///
/// Internal use: used to track the hydroflow graph structure.
///
/// TODO(mingwei): restructure `PortList` so this can be crate-private.
pub struct HandoffData {
    /// A friendly name for diagnostics.
    #[allow(dead_code)] // TODO(mingwei): remove attr once used.
    name: Cow<'static, str>,
    /// Crate-visible to crate for `handoff_list` internals.
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
    pub fn new(name: Cow<'static, str>, handoff: impl 'static + HandoffMeta) -> Self {
        let (preds, succs) = Default::default();
        Self {
            name,
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
    /// A friendly name for diagnostics.
    #[allow(dead_code)] // TODO(mingwei): remove attr once used.
    name: Cow<'static, str>,
    /// This subgraph's stratum number.
    stratum: usize,
    /// The actual execution code of the subgraph.
    subgraph: Box<dyn Subgraph>,
    #[allow(dead_code)]
    preds: Vec<HandoffId>,
    succs: Vec<HandoffId>,
    dependencies: DirectedEdgeSet,
    /// If this subgraph is scheduled in [`Hydroflow::stratum_queues`].
    /// [`Cell`] allows modifying this field when iterating `Self::preds` or
    /// `Self::succs`, as all `SubgraphData` are owned by the same vec
    /// `Hydroflow::subgraphs`.
    is_scheduled: Cell<bool>,
}
impl SubgraphData {
    pub fn new(
        name: Cow<'static, str>,
        stratum: usize,
        subgraph: impl 'static + Subgraph,
        preds: Vec<HandoffId>,
        succs: Vec<HandoffId>,
        dependencies: DirectedEdgeSet,
        is_scheduled: bool,
    ) -> Self {
        Self {
            name,
            stratum,
            subgraph: Box::new(subgraph),
            preds,
            succs,
            dependencies,
            is_scheduled: Cell::new(is_scheduled),
        }
    }
}

/// Internal struct containing a pointer to [`Hydroflow`]-owned state.
pub(crate) struct StateData {
    pub state: Box<dyn Any>,
}
