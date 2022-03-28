use std::any::Any;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::{HashMap, HashSet, VecDeque};
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
use serde::Serialize;

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
                let sg_data = &mut self.subgraphs[sg_id.0];
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

            for &handoff_id in self.subgraphs[sg_id.0].succs.iter() {
                let handoff = &self.handoffs[handoff_id.0];
                if !handoff.handoff.is_bottom() {
                    for &succ_id in handoff.succs.iter() {
                        let succ_sg_data = &self.subgraphs[succ_id.0];
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
            let sg_data = &self.subgraphs[sg_id.0];
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
            let sg_data = &self.subgraphs[sg_id.0];
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
            let sg_data = &self.subgraphs[sg_id.0];
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
        let sg_id = SubgraphId(self.subgraphs.len());

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
            FlowGraph::default(),
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
        let sg_id = SubgraphId(self.subgraphs.len());

        let subgraph_preds = recv_ports.iter().map(|port| port.handoff_id).collect();
        let subgraph_succs = send_ports.iter().map(|port| port.handoff_id).collect();

        for recv_port in recv_ports.iter() {
            self.handoffs[recv_port.handoff_id.0].succs.push(sg_id);
        }
        for send_port in send_ports.iter() {
            self.handoffs[send_port.handoff_id.0].preds.push(sg_id);
        }

        let subgraph = move |context: Context<'_>| {
            let recvs: Vec<&RecvCtx<R>> = recv_ports
                .iter()
                .map(|hid| hid.handoff_id)
                .map(|hid| context.handoffs.get(hid.0).unwrap())
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
                .map(|hid| context.handoffs.get(hid.0).unwrap())
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
            FlowGraph::default(),
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
        let handoff_id = HandoffId(self.handoffs.len());

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

    pub fn add_dependencies(&mut self, sg_id: SubgraphId, deps: FlowGraph) {
        self.subgraphs[sg_id.0].dependencies.append(deps);
    }

    fn mermaid_mangle(
        &self,
        name: Cow<'static, str>,
        sg_id: SubgraphId,
        node_id: NodeId,
    ) -> String {
        if let Some(handoff_id) = self.subgraphs[sg_id.0]
            .dependencies
            .handoff_ids
            .get(&node_id)
        {
            let handoff = &self.handoffs[handoff_id.0];
            format!("Handoff_{}[\\{}/]", handoff_id.0, handoff.name)
        } else if &*name == "PullToPush" {
            format!("{}.{}[/{}\\]", sg_id.0, node_id.0, name)
        } else {
            format!("{}.{}[{}]", sg_id.0, node_id.0, name)
        }
    }

    pub fn generate_mermaid(&self) -> String {
        let mut output = String::new();
        self.write_mermaid(&mut output).unwrap();
        output
    }

    pub fn write_mermaid(&self, write: &mut impl Write) -> std::fmt::Result {
        writeln!(write, "graph TD")?;
        let mut tab: usize = 2;
        for (sg_id, sg) in self.subgraphs.iter().enumerate() {
            let sg_id = SubgraphId(sg_id);
            let d = &sg.dependencies;

            if !d.edges.is_empty() {
                writeln!(write, "{:t$}subgraph stratum{}", "", sg.stratum, t = tab)?;
                tab += 2;
                writeln!(write, "{:t$}subgraph {}{}", "", sg.name, sg_id.0, t = tab)?;
                tab += 2;
                for &(src, dst) in d.edges.iter() {
                    let src = self.mermaid_mangle(d.node_names[src.0].clone(), sg_id, src);
                    let dst = self.mermaid_mangle(d.node_names[dst.0].clone(), sg_id, dst);
                    writeln!(write, "{:t$}{} --> {}", "", src, dst, t = tab)?;
                }
                tab -= 2;
                writeln!(write, "{:t$}end", "", t = tab)?;
                tab -= 2;
                writeln!(write, "{:t$}end", "", t = tab)?;
            }
        }
        Ok(())
    }

    pub fn generate_dot(&self) -> String {
        let mut output = String::new();
        self.write_dot(&mut output).unwrap();
        output
    }

    pub fn write_dot(&self, w: &mut impl Write) -> std::fmt::Result {
        let gg = self.global_graph();
        writeln!(w, "digraph {{")?;
        let mut tab: usize = 2;
        // write out nodes
        writeln!(w, "{:t$}{{", "", t = tab)?;
        tab += 2;
        writeln!(w, "{:t$}node [shape=box]", "", t = tab)?;
        for (i, name) in gg.node_names.iter().enumerate() {
            if name.is_some() {
                let nm = name.clone().unwrap_or_else(|| "".into());
                let label = format!("{}", i);
                write!(w, "{:t$}{} [label=\"{}\"", "", label, nm.clone(), t = tab)?;
                if nm.starts_with("Handoff") {
                    write!(w, ", shape=invtrapezium")?;
                } else if nm == "PullToPush" {
                    write!(w, ", shape=trapezium")?;
                }
                writeln!(w, "]")?;
            }
        }
        tab -= 2;
        writeln!(w, "{:t$}}}", "", t = tab)?;

        // write out edges
        for (sg_id, es) in gg.edge_sets.iter().enumerate() {
            let strt = es.stratum;
            writeln!(
                w,
                "{:t$}subgraph \"cluster stratum {}\" {{",
                "",
                strt,
                t = tab,
            )?;
            tab += 2;
            writeln!(w, "{:t$}label = \"Stratum {}\"", "", strt, t = tab,)?;
            writeln!(w, "{:t$}subgraph \"cluster {}\" {{", "", sg_id, t = tab,)?;
            tab += 2;
            writeln!(w, "{:t$}label = \"{}\"", "", es.name, t = tab)?;
            for &(src, dst) in es.edges.iter() {
                writeln!(w, "{:t$}{} -> {}", "", src.0, dst.0, t = tab,)?;
            }
            tab -= 2;
            writeln!(w, "{:t$}}}", "", t = tab)?;
            tab -= 2;
            writeln!(w, "{:t$}}}", "", t = tab)?;
        }
        tab -= 2;
        writeln!(w, "{:t$}}}", "", t = tab)?;
        Ok(())
    }

    pub fn generate_json(&self) -> String {
        let mut output = String::new();
        self.write_json(&mut output).unwrap();
        output
    }

    pub fn write_json(&self, write: &mut impl Write) -> std::fmt::Result {
        writeln!(
            write,
            "{}",
            serde_json::to_string(&self.global_graph()).unwrap()
        )?;
        Ok(())
    }

    pub fn global_graph(&self) -> GlobalGraph {
        let mut graph = GlobalGraph::new();
        for subgraph in self.subgraphs.iter() {
            if !subgraph.dependencies.edges.is_empty() {
                graph.add_flow_graph(
                    subgraph.name.clone(),
                    subgraph.stratum,
                    subgraph.dependencies.clone(),
                    &self.handoffs,
                );
            }
        }
        graph.canonicalize_handoffs();
        graph
    }
}
/// A FlowGraph nodes's ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[repr(transparent)]
pub struct NodeId(pub(crate) usize);

#[derive(Clone, Debug, Default, Serialize)]
pub struct EdgeSet {
    name: Cow<'static, str>,
    stratum: usize,
    edges: HashSet<(NodeId, NodeId)>,
}
impl EdgeSet {
    pub fn new(name: Cow<'static, str>, stratum: usize, edges: HashSet<(NodeId, NodeId)>) -> Self {
        Self {
            name,
            stratum,
            edges,
        }
    }
}
// A graph connecting up multiple compiled components
#[derive(Clone, Debug, Default, Serialize)]
pub struct GlobalGraph {
    node_names: Vec<Option<Cow<'static, str>>>,
    edge_sets: Vec<EdgeSet>,
    handoff_ids: HashMap<NodeId, HandoffId>,
}
impl GlobalGraph {
    pub fn new() -> Self {
        let (node_names, edge_sets, handoff_ids) = Default::default();
        Self {
            node_names,
            edge_sets,
            handoff_ids,
        }
    }
    // handoff names are repeated across compiled components
    // make sure all edges reference the first occurrence of a handoff
    // and then None out the repeated occurrences.
    // Preferably to be called only once after all FlowGraphs are added.
    fn canonicalize_handoffs(&mut self) {
        // invert self.handoff_ids
        let mut handoff_ids_inv: HashMap<HandoffId, Vec<NodeId>> = HashMap::new();
        for (k, v) in &self.handoff_ids {
            handoff_ids_inv.entry(*v).or_insert_with(Vec::new).push(*k);
        }

        // find repeated handoffs
        let mut repeated_handoffs: Vec<NodeId> = Vec::new();
        for w in handoff_ids_inv.values() {
            repeated_handoffs.extend(w.clone().split_off(1));
        }

        // walk edges and replace every handoff ref with the one canonical ref
        // taken from handoff_ids_inv[id].first()
        let mut new_edge_sets = Vec::new();
        for es in self.edge_sets.iter() {
            let mut new_edges = HashSet::new();
            for (from, to) in es.edges.iter() {
                let mut new_edge = (*from, *to);
                let hoff_id = self.handoff_ids.get(from);
                if let Some(id) = hoff_id {
                    new_edge.0 = *handoff_ids_inv[id].first().unwrap();
                }
                let hoff_id = self.handoff_ids.get(to);
                if let Some(id) = hoff_id {
                    new_edge.1 = *handoff_ids_inv[id].first().unwrap();
                }
                new_edges.insert(new_edge);
            }
            let new_es = EdgeSet::new(es.name.clone(), es.stratum, new_edges);
            new_edge_sets.push(new_es);
        }
        self.edge_sets = new_edge_sets;

        // None out the repeated_handoffs in node_names
        for n in repeated_handoffs {
            self.node_names[n.0] = None;
        }
    }

    pub fn add_flow_graph(
        &mut self,
        name: Cow<'static, str>,
        stratum: usize,
        fg: FlowGraph,
        handoffs: &[HandoffData],
    ) {
        let base = self.node_names.len();
        for node_name in fg.node_names {
            self.node_names.push(Some(node_name));
        }
        let edges: HashSet<(NodeId, NodeId)> = fg
            .edges
            .into_iter()
            .map(|(from, to)| (NodeId(from.0 + base), NodeId(to.0 + base)))
            .collect();
        self.edge_sets
            .push(EdgeSet::new(name.clone(), stratum, edges));
        for (node_id, hoff_id) in fg.handoff_ids.iter() {
            self.node_names[node_id.0 + base] =
                Some(format!("{}", handoffs[hoff_id.0].name).into());
            self.handoff_ids
                .entry(NodeId(node_id.0 + base))
                .or_insert(*hoff_id);
        }
    }
}

/// A graph representation of a compiled component's graph structure.
#[derive(Clone, Debug, Default, Serialize)]
pub struct FlowGraph {
    node_names: Vec<Cow<'static, str>>,
    edges: HashSet<(NodeId, NodeId)>,
    handoff_ids: HashMap<NodeId, HandoffId>,
}

impl FlowGraph {
    pub fn new() -> Self {
        let (node_names, edges, handoff_ids) = Default::default();
        Self {
            node_names,
            edges,
            handoff_ids,
        }
    }

    pub fn add_node(&mut self, node_info: impl Into<Cow<'static, str>>) -> NodeId {
        let current_index = self.node_names.len();
        self.node_names.insert(current_index, node_info.into());
        NodeId(current_index)
    }

    pub fn add_edge(&mut self, edge: (NodeId, NodeId)) {
        self.edges.insert(edge);
    }

    pub fn add_handoff_id(&mut self, node_id: NodeId, handoff_id: HandoffId) {
        self.handoff_ids.insert(node_id, handoff_id);
    }

    pub fn append(&mut self, mut other: FlowGraph) {
        let base = self.node_names.len();
        self.node_names.append(&mut other.node_names);
        self.edges.extend(
            other
                .edges
                .into_iter()
                .map(|(from, to)| (NodeId(from.0 + base), NodeId(to.0 + base))),
        );
        for (node_id, hoff_id) in other.handoff_ids.iter() {
            self.handoff_ids
                .entry(NodeId(node_id.0 + base))
                .or_insert(*hoff_id);
        }
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
    dependencies: FlowGraph,
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
        dependencies: FlowGraph,
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
