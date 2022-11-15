use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use serde::Serialize;

use super::graph::HandoffData;
use super::HandoffId;
/// A FlowGraph nodes's ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[repr(transparent)]
pub struct FlowNodeId(pub(crate) usize);

#[derive(Clone, Debug, Default, Serialize)]
pub struct FlowEdgeSet {
    name: Cow<'static, str>,
    stratum: usize,
    edges: HashSet<(FlowNodeId, FlowNodeId)>,
}
impl FlowEdgeSet {
    pub fn new(
        name: Cow<'static, str>,
        stratum: usize,
        edges: HashSet<(FlowNodeId, FlowNodeId)>,
    ) -> Self {
        Self {
            name,
            stratum,
            edges,
        }
    }
}
// A graph connecting up multiple compiled components
#[derive(Clone, Debug, Default, Serialize)]
pub struct FlowPartitionedGraph {
    node_names: Vec<Option<Cow<'static, str>>>,
    edge_sets: Vec<FlowEdgeSet>,
    handoff_ids: HashMap<FlowNodeId, HandoffId>,
}
impl FlowPartitionedGraph {
    pub fn new() -> Self {
        let (node_names, edge_sets, handoff_ids) = Default::default();
        Self {
            node_names,
            edge_sets,
            handoff_ids,
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
        let edges: HashSet<(FlowNodeId, FlowNodeId)> = fg
            .edges
            .into_iter()
            .map(|(from, to)| (FlowNodeId(from.0 + base), FlowNodeId(to.0 + base)))
            .collect();
        self.edge_sets
            .push(FlowEdgeSet::new(name.clone(), stratum, edges));
        for (node_id, hoff_id) in fg.handoff_ids.iter() {
            self.node_names[node_id.0 + base] =
                Some(format!("{}", handoffs[hoff_id.0].name).into());
            self.handoff_ids
                .entry(FlowNodeId(node_id.0 + base))
                .or_insert(*hoff_id);
        }
    }
}

/// A graph representation of a compiled component's graph structure.
#[derive(Clone, Debug, Default, Serialize)]
pub struct FlowGraph {
    node_names: Vec<Cow<'static, str>>,
    edges: HashSet<(FlowNodeId, FlowNodeId)>,
    handoff_ids: HashMap<FlowNodeId, HandoffId>,
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

    pub fn add_node(&mut self, node_info: impl Into<Cow<'static, str>>) -> FlowNodeId {
        let current_index = self.node_names.len();
        self.node_names.insert(current_index, node_info.into());
        FlowNodeId(current_index)
    }

    pub fn add_edge(&mut self, edge: (FlowNodeId, FlowNodeId)) {
        self.edges.insert(edge);
    }

    pub fn add_handoff_id(&mut self, node_id: FlowNodeId, handoff_id: HandoffId) {
        self.handoff_ids.insert(node_id, handoff_id);
    }

    pub fn append(&mut self, mut other: FlowGraph) {
        let base = self.node_names.len();
        self.node_names.append(&mut other.node_names);
        self.edges.extend(
            other
                .edges
                .into_iter()
                .map(|(from, to)| (FlowNodeId(from.0 + base), FlowNodeId(to.0 + base))),
        );
        for (node_id, hoff_id) in other.handoff_ids.iter() {
            self.handoff_ids
                .entry(FlowNodeId(node_id.0 + base))
                .or_insert(*hoff_id);
        }
    }
}
