use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use serde::Serialize;

use super::graph::{HandoffData, Hydroflow};
use super::{HandoffId, SubgraphId};

impl Hydroflow {
    fn mermaid_mangle(
        &self,
        name: Cow<'static, str>,
        sg_id: SubgraphId,
        node_id: FlowNodeId,
    ) -> String {
        if let Some(handoff_id) = self.subgraphs[sg_id.0]
            .dependencies
            .handoff_ids
            .get(&node_id)
        {
            let handoff = &self.context.handoffs[handoff_id.0];
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

    pub fn global_graph(&self) -> FlowPartitionedGraph {
        let mut graph = FlowPartitionedGraph::new();
        for subgraph in self.subgraphs.iter() {
            if !subgraph.dependencies.edges.is_empty() {
                graph.add_flow_graph(
                    subgraph.name.clone(),
                    subgraph.stratum,
                    subgraph.dependencies.clone(),
                    &*self.context.handoffs,
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
    // handoff names are repeated across compiled components
    // make sure all edges reference the first occurrence of a handoff
    // and then None out the repeated occurrences.
    // Preferably to be called only once after all FlowGraphs are added.
    fn canonicalize_handoffs(&mut self) {
        // invert self.handoff_ids
        let mut handoff_ids_inv: HashMap<HandoffId, Vec<FlowNodeId>> = HashMap::new();
        for (k, v) in &self.handoff_ids {
            handoff_ids_inv.entry(*v).or_insert_with(Vec::new).push(*k);
        }

        // find repeated handoffs
        let mut repeated_handoffs: Vec<FlowNodeId> = Vec::new();
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
            let new_es = FlowEdgeSet::new(es.name.clone(), es.stratum, new_edges);
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
