//! Module for determining flow properties. See [`propegate_flow_props`].

use super::{HydroflowGraph, Node};
use crate::graph::graph_algorithms;

/// Traverses the graph, propegating the flow properties from sources to sinks.
pub fn propegate_flow_props(graph: &mut HydroflowGraph) {
    let node_order = graph_algorithms::topo_sort_scc(
        || graph.node_ids(),
        |dst| graph.node_predecessor_nodes(dst),
        |src| graph.node_successor_nodes(src),
    );
    for (idx_star_ord, node_id) in node_order.into_iter().enumerate() {
        match graph.node(node_id) {
            Node::Operator(_) => {
                let op_inst = graph
                    .node_op_inst(node_id)
                    .expect("Operator instance info must be set when calling `propegate_flow_props`. (This is a Hydroflow bug).");

                if let Some(flow_prop_fn) = op_inst.op_constraints.flow_prop_fn {
                    let flow_props_in = graph
                        .node_predecessor_edges(node_id)
                        .map(|edge_id| graph.edge_flow_props(edge_id))
                        .collect::<Vec<_>>();

                    let flow_props_out = (flow_prop_fn)(&flow_props_in, op_inst, idx_star_ord);
                    assert!(
                        1 == flow_props_out.len()
                            || graph.node_degree_out(node_id) == flow_props_out.len()
                    );

                    let out_edges = graph.node_successor_edges(node_id).collect::<Vec<_>>();
                    // In/out edges are in the same order as the in/out port names (in `op_inst`).
                    for (i, edge_id) in out_edges.into_iter().enumerate() {
                        if let Some(flow_prop_out) = *flow_props_out
                            .get(i)
                            .unwrap_or_else(|| flow_props_out.get(0).unwrap())
                        {
                            graph.set_edge_flow_props(edge_id, flow_prop_out);
                        }
                    }
                }
            }
            Node::Handoff { .. } => {
                assert_eq!(1, graph.node_degree_in(node_id));
                assert_eq!(1, graph.node_degree_out(node_id));
                let in_edge = graph.node_predecessor_edges(node_id).next().unwrap();
                let out_edge = graph.node_successor_edges(node_id).next().unwrap();
                if let Some(flow_props) = graph.edge_flow_props(in_edge) {
                    graph.set_edge_flow_props(out_edge, flow_props);
                }
            }
        }
    }
}
