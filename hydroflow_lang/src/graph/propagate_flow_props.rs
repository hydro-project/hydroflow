//! Module for determining flow properties. See [`propagate_flow_props`].

use super::ops::_upcast::_UPCAST;
use super::{GraphNodeId, HydroflowGraph, Node};
use crate::diagnostic::Diagnostic;
use crate::graph::graph_algorithms;
use crate::graph::ops::FlowPropArgs;

/// Traverses the graph, propagating the flow properties from sources to sinks.
pub fn propagate_flow_props(
    graph: &mut HydroflowGraph,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), GraphNodeId> {
    // Topological sort on strongly connected components.
    // TODO(mingwei): order is broken within SCCs/cycles: https://github.com/hydro-project/hydroflow/issues/895
    let mut node_order = graph_algorithms::topo_sort_scc(
        || graph.node_ids(),
        |dst| graph.node_predecessor_nodes(dst),
        |src| graph.node_successor_nodes(src),
    );
    // Only retain nodes that have a `flow_prop_fn` or are handoffs.
    node_order.retain(|&node_id| match graph.node(node_id) {
        Node::Operator(_) => graph
            .node_op_inst(node_id)
            .and_then(|op_inst| op_inst.op_constraints.flow_prop_fn)
            .is_some(),
        Node::Handoff { .. } => true,
        Node::ModuleBoundary { .. } => panic!(),
    });
    // Put upcast nodes first.
    node_order.sort_by_key(|&node_id| {
        graph
            .node_op_inst(node_id)
            .map_or(true, |op_inst| op_inst.op_constraints.name != _UPCAST.name)
    });

    // Propagate flow props in order.
    loop {
        let mut changed = false;
        for (idx_star_ord, &node_id) in node_order.iter().enumerate() {
            match graph.node(node_id) {
                Node::Operator(_) => {
                    let op_inst = graph.node_op_inst(node_id)
                        .expect("Operator instance info must be set when calling `propagate_flow_props`. (This is a Hydroflow bug).");

                    if let Some(flow_prop_fn) = op_inst.op_constraints.flow_prop_fn {
                        // Collect the flow props on input edges. Input operators will naturally have no inputs.
                        let flow_props_in = graph
                            .node_predecessor_edges(node_id)
                            .map(|edge_id| graph.edge_flow_props(edge_id))
                            .collect::<Vec<_>>();

                        // Build the `FlowPropArgs` argument
                        let flow_prop_args = FlowPropArgs {
                            op_span: graph.node(node_id).span(),
                            op_name: op_inst.op_constraints.name,
                            op_inst,
                            flow_props_in: &flow_props_in,
                            new_star_ord: idx_star_ord,
                        };

                        // Call the `flow_prop_fn`.
                        // TODO(mingwei): don't exit early here?
                        let flow_props_out =
                            (flow_prop_fn)(flow_prop_args, diagnostics).map_err(|()| node_id)?;
                        assert!(
                            1 == flow_props_out.len()
                                || graph.node_degree_out(node_id) == flow_props_out.len()
                        );

                        // Assign output flow props.
                        let out_edges = graph.node_successor_edges(node_id).collect::<Vec<_>>();
                        // In/out edges are in the same order as the in/out port names (in `op_inst`).
                        for (i, edge_id) in out_edges.into_iter().enumerate() {
                            if let Some(flow_prop_out) = *flow_props_out
                                .get(i)
                                .unwrap_or_else(|| flow_props_out.first().unwrap())
                            {
                                let flow_prop_old =
                                    graph.set_edge_flow_props(edge_id, flow_prop_out);
                                changed |= flow_prop_old.map_or(true, |old| old != flow_prop_out);
                            }
                        }
                    }
                }
                Node::Handoff { .. } => {
                    // Handoffs just copy over their one input [`FlowProps`] to their one output.
                    assert_eq!(1, graph.node_degree_in(node_id));
                    assert_eq!(1, graph.node_degree_out(node_id));
                    let in_edge = graph.node_predecessor_edges(node_id).next().unwrap();
                    let out_edge = graph.node_successor_edges(node_id).next().unwrap();
                    if let Some(flow_props) = graph.edge_flow_props(in_edge) {
                        graph.set_edge_flow_props(out_edge, flow_props);
                    }
                }
                _ => {
                    // If a module boundary is encountered then something has gone wrong.
                    panic!();
                }
            }
        }
        if !changed {
            break;
        }
    }
    Ok(())
}
