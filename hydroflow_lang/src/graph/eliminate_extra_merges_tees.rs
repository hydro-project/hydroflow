#![deny(missing_docs)]

use super::{GraphNodeId, HydroflowGraph};

fn find_unary_ops<'a>(
    graph: &'a HydroflowGraph,
    op_name: &'static str,
) -> impl 'a + Iterator<Item = GraphNodeId> {
    graph
        .node_ids()
        .filter(move |&node_id| {
            graph
                .node_op_inst(node_id)
                .map_or(false, |op_inst| op_name == op_inst.op_constraints.name)
        })
        .filter(|&node_id| {
            1 == graph.node_degree_in(node_id) && 1 == graph.node_degree_out(node_id)
        })
}

/// Removes missing merges and tees. Must be applied BEFORE subgraph partitioning.
pub fn eliminate_extra_merges_tees(graph: &mut HydroflowGraph) {
    let extra_ops = find_unary_ops(graph, "merge")
        .chain(find_unary_ops(graph, "tee"))
        .collect::<Vec<_>>();
    for extra_op in extra_ops {
        graph.remove_intermediate_node(extra_op);
    }
}
