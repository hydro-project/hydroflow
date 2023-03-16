use std::collections::BTreeMap;

use slotmap::{SecondaryMap, SlotMap, SparseSecondaryMap};

use serde::{Deserialize, Serialize};

use super::{
    graph_write::{Dot, GraphWrite, Mermaid},
    node_color,
    ops::DelayType,
    Color, GraphNodeId, GraphSubgraphId, HANDOFF_NODE_STR,
};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct SerdeEdge {
    pub src: GraphNodeId,
    pub dst: GraphNodeId,
    pub blocking: bool,
    pub label: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SerdeGraph {
    pub nodes: SecondaryMap<GraphNodeId, String>,
    pub edges: SecondaryMap<GraphNodeId, Vec<SerdeEdge>>,
    pub barrier_handoffs: SparseSecondaryMap<GraphNodeId, bool>,
    pub subgraph_nodes: SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
    pub subgraph_stratum: SecondaryMap<GraphSubgraphId, usize>,
    pub subgraph_internal_handoffs: SecondaryMap<GraphSubgraphId, Vec<GraphNodeId>>,

    /// What variable name each graph node belongs to (if any).
    /// The nodes that each variable name encompases.
    pub varname_nodes: BTreeMap<String, Vec<GraphNodeId>>,
}

impl SerdeGraph {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn to_mermaid(&self) -> String {
        let mut output = String::new();
        self.write_mermaid(&mut output).unwrap();
        output
    }

    pub fn write_mermaid(&self, output: impl std::fmt::Write) -> std::fmt::Result {
        let mut graph_write = Mermaid::new(output);
        self.write_graph(&mut graph_write)
    }

    pub fn to_dot(&self) -> String {
        let mut output = String::new();
        let mut graph_write = Dot::new(&mut output);
        self.write_graph(&mut graph_write).unwrap();
        output
    }

    pub fn write_dot(&self, output: impl std::fmt::Write) -> std::fmt::Result {
        let mut graph_write = Dot::new(output);
        self.write_graph(&mut graph_write)
    }

    pub fn write_graph<W>(&self, mut graph_write: W) -> Result<(), W::Err>
    where
        W: GraphWrite,
    {
        graph_write.write_prologue()?;

        let node_color_map = self.make_node_color_map();
        for (subgraph_id, node_ids) in self.subgraph_nodes.iter() {
            let stratum = self.subgraph_stratum.get(subgraph_id);
            graph_write.write_subgraph_start(subgraph_id, *stratum.unwrap())?;

            // write out nodes
            for &node_id in node_ids.iter() {
                graph_write.write_node(
                    node_id,
                    &**self.nodes.get(node_id).unwrap(),
                    node_color_map.get(node_id).copied().unwrap_or(Color::Comp),
                    Some(subgraph_id),
                )?;
            }
            // write out internal handoffs
            let empty = vec![];
            if let Some(hoffs) = self.subgraph_internal_handoffs.get(subgraph_id) {
                for &hoff in hoffs.iter() {
                    graph_write.write_node(
                        hoff,
                        &**self.nodes.get(hoff).unwrap(),
                        Color::Hoff,
                        Some(subgraph_id),
                    )?;
                    // write out internal handoff edges
                    for edge in self.edges.get(hoff).unwrap_or(&empty) {
                        // TODO(mingwei): not precise
                        let delay_type = if edge.blocking {
                            Some(DelayType::Stratum)
                        } else {
                            None
                        };
                        graph_write.write_edge(
                            edge.src,
                            edge.dst,
                            delay_type,
                            edge.label.as_deref(),
                            Some(subgraph_id),
                        )?;
                    }
                }
            }

            // write out edges
            for &src in node_ids.iter() {
                if let Some(edges) = self.edges.get(src) {
                    for edge in edges {
                        if !self.barrier_handoffs.contains_key(edge.dst) {
                            // TODO(mingwei): not precise
                            let delay_type = if edge.blocking {
                                Some(DelayType::Stratum)
                            } else {
                                None
                            };
                            graph_write.write_edge(
                                edge.src,
                                edge.dst,
                                delay_type,
                                edge.label.as_deref(),
                                Some(subgraph_id),
                            )?;
                        }
                    }
                }
            }

            // write out any variable names
            for (varname, varname_node_ids) in self.varname_nodes.iter() {
                // TODO(mingwei): this is awkward, inefficient runtime
                let mut varname_nodes = varname_node_ids
                    .iter()
                    .copied()
                    .filter(|node_id| node_ids.contains(node_id))
                    .peekable();
                if varname_nodes.peek().is_some() {
                    graph_write.write_subgraph_varname(subgraph_id, varname, varname_nodes)?;
                }
            }

            graph_write.write_subgraph_end()?;
        }

        //write out handoffs outside the clusters and adjacent edges
        for (src, edges) in self.edges.iter() {
            for edge in edges {
                // TODO(mingwei): not precise
                let delay_type = if edge.blocking {
                    Some(DelayType::Stratum)
                } else {
                    None
                };
                if self.barrier_handoffs.contains_key(src) {
                    // write out handoff
                    graph_write.write_node(src, &*self.nodes[src], Color::Hoff, None)?;
                    // write out edge
                    graph_write.write_edge(
                        src,
                        edge.dst,
                        delay_type,
                        edge.label.as_deref(),
                        None,
                    )?;
                } else if self.barrier_handoffs.contains_key(edge.dst) {
                    // just write out edge
                    graph_write.write_edge(
                        src,
                        edge.dst,
                        delay_type,
                        edge.label.as_deref(),
                        None,
                    )?;
                }
            }
        }

        graph_write.write_epilogue()?;

        Ok(())
    }

    fn make_node_color_map(&self) -> SparseSecondaryMap<GraphNodeId, Color> {
        // TODO(mingwei): this repeated code will be unified when `SerdeGraph` is subsumed into `HydroflowGraph`.
        // TODO(mingwei): REPEATED CODE, COPIED FROM `flat_to_partitioned.rs`
        // but modified for `serde_graph`...
        let mut inn_degree: SecondaryMap<GraphNodeId, usize> =
            SecondaryMap::with_capacity(self.nodes.len());
        for edge in self.edges.values().flatten() {
            *inn_degree.entry(edge.dst).unwrap().or_insert(0) += 1;
        }

        let mut node_color_map: SparseSecondaryMap<GraphNodeId, Color> = self
            .nodes
            .iter()
            .filter_map(|(node_id, node_str)| {
                let inn_degree = inn_degree.get(node_id).copied().unwrap_or(0);
                let out_degree = self.edges.get(node_id).map(Vec::len).unwrap_or(0);
                let is_handoff = HANDOFF_NODE_STR == node_str;
                let op_color = node_color(is_handoff, inn_degree, out_degree);
                op_color.map(|op_color| (node_id, op_color))
            })
            .collect();

        // Fill in rest via subgraphs.
        for sg_nodes in self.subgraph_nodes.values() {
            // TODO(mingwei): REPEATED CODE, COPIED FROM `partitioned_graph.rs` codegen.
            let pull_to_push_idx = sg_nodes
                .iter()
                .position(|&node_id| {
                    let inn_degree = inn_degree.get(node_id).copied().unwrap_or(0);
                    let out_degree = self.edges.get(node_id).map(Vec::len).unwrap_or(0);
                    let node_str = &self.nodes[node_id];
                    let is_handoff = HANDOFF_NODE_STR == node_str;
                    node_color(is_handoff, inn_degree, out_degree)
                        .map(|color| Color::Pull != color)
                        .unwrap_or(false)
                })
                .unwrap_or(sg_nodes.len());

            for (idx, node_id) in sg_nodes.iter().copied().enumerate() {
                let is_pull = idx < pull_to_push_idx;
                node_color_map.insert(node_id, if is_pull { Color::Pull } else { Color::Push });
            }
        }

        node_color_map
    }
}
