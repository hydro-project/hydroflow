//! Module for determining flow properties. See [`propegate_flow_props`].

use super::HydroflowGraph;
use crate::graph::graph_algorithms;

/// Traverses the graph, propegating the flow properties from sources to sinks.
pub fn propegate_flow_props(graph: &mut HydroflowGraph) {
    let node_order = graph_algorithms::topo_sort_scc(
        || graph.node_ids(),
        |dst| graph.node_predecessor_nodes(dst),
        |src| graph.node_successor_nodes(src),
    );
    for (idx_star_ord, node_id) in node_order.into_iter().enumerate() {
        if let Some(flow_prop_fn) = graph
            .node_op_inst(node_id)
            .expect("OperatorInstance must be set.")
            .op_constraints
            .flow_prop_fn
        {
            let flow_props_in = graph
                .node_predecessor_edges(node_id)
                .map(|edge_id| graph.edge_flow_props(edge_id))
                .collect::<Vec<_>>();
            let op_inst = graph
                .node_op_inst(node_id)
                .expect("Operator instance info must be set when calling `propegate_flow_props`.");

            let flow_props_out = (flow_prop_fn)(&flow_props_in, op_inst, idx_star_ord);
            assert!(
                1 == flow_props_out.len() || graph.node_degree_out(node_id) == flow_props_out.len()
            );

            let out_edges = graph.node_successor_edges(node_id).collect::<Vec<_>>();
            // TODO(mingwei): SORT EDGES DETERMINISTICALLY BY NAME (SOMEHOW?).
            for (i, edge_id) in out_edges.into_iter().enumerate() {
                graph.set_edge_flow_props(
                    edge_id,
                    *flow_props_out
                        .get(i)
                        .unwrap_or_else(|| flow_props_out.get(0).unwrap()),
                );
            }
        }
    }
}
