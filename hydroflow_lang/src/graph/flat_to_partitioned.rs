use std::collections::{HashMap, HashSet};

use proc_macro2::Span;
use slotmap::{Key, SecondaryMap, SlotMap};
use syn::{parse_quote, spanned::Spanned};

use crate::{parse::IndexInt, union_find::UnionFind};

use super::{
    flat_graph::FlatGraph,
    graph_algorithms, iter_edges, node_color,
    ops::{DelayType, OPERATORS},
    partitioned_graph::PartitionedGraph,
    AdjList, Color, EdgePort, GraphNodeId, GraphSubgraphId, Node,
};

fn find_barrier_crossers(
    nodes: &SlotMap<GraphNodeId, Node>,
    succs: &AdjList,
) -> Vec<(GraphNodeId, GraphNodeId, IndexInt, DelayType)> {
    iter_edges(succs)
        .filter_map(|((src, _src_idx), (dst, dst_idx))| {
            if let Node::Operator(dst_operator) = &nodes[dst] {
                let dst_name = &*dst_operator.name_string();
                OPERATORS
                    .iter()
                    .find(|&op| dst_name == op.name)
                    .and_then(|op_constraints| (op_constraints.input_delaytype_fn)(dst_idx.value))
                    .map(|input_barrier| (src, dst, *dst_idx, input_barrier))
            } else {
                None
            }
        })
        .collect()
}

fn find_subgraph_unionfind(
    nodes: &mut SlotMap<GraphNodeId, Node>,
    preds: &mut AdjList,
    succs: &mut AdjList,
    barrier_crossers: &[(GraphNodeId, GraphNodeId, IndexInt, DelayType)],
) -> (UnionFind<GraphNodeId>, HashSet<(EdgePort, EdgePort)>) {
    // Pre-calculate node colors.
    let mut node_color: SecondaryMap<GraphNodeId, Option<Color>> = nodes
        .keys()
        .map(|node_id| {
            let inn_degree = preds[node_id].len();
            let out_degree = succs[node_id].len();
            let op_color = node_color(&nodes[node_id], inn_degree, out_degree);
            (node_id, op_color)
        })
        .collect();

    let mut subgraph_unionfind: UnionFind<GraphNodeId> = UnionFind::with_capacity(nodes.len());
    // All edges which are handoffs. Starts out with all edges and we remove
    // from this set as we construct subgraphs.
    let mut handoff_edges: HashSet<(EdgePort, EdgePort)> = iter_edges(succs)
        .map(|((src, &src_idx), (dst, &dst_idx))| ((src, src_idx), (dst, dst_idx)))
        .collect();

    // Sort edges here (for now, no sort/priority).
    for ((src, src_idx), (dst, dst_idx)) in iter_edges(succs) {
        // (Each edge gets looked at once to check if it can be unioned into one subgraph.)

        // Ignore (1) already added edges as well as (2) new self-cycles.
        if subgraph_unionfind.same_set(src, dst) {
            // Note this might be triggered even if the edge (src, dst) is not in the subgraph (not case 1).
            // This prevents self-loops which would violate the in-out tree structure (case 2).
            // Handoffs will be inserted later for this self-loop.
            continue;
        }

        // Ignore if would join stratum crossers (next edges).
        if barrier_crossers
            .iter()
            .any(|&(x_src, x_dst, _x_dst_idx, _x_input_barrier)| {
                (subgraph_unionfind.same_set(x_src, src) && subgraph_unionfind.same_set(x_dst, dst))
                    || (subgraph_unionfind.same_set(x_src, dst)
                        && subgraph_unionfind.same_set(x_dst, src))
            })
        {
            continue;
        }

        if can_connect_colorize(&mut node_color, src, dst) {
            // At this point we have selected this edge and its src & dst to be
            // within a single subgraph.
            subgraph_unionfind.union(src, dst);
            assert!(handoff_edges.remove(&((src, *src_idx), (dst, *dst_idx))));
        }
    }
    (subgraph_unionfind, handoff_edges)
}

fn find_subgraph_collect(
    nodes: &mut SlotMap<GraphNodeId, Node>,
    preds: &mut AdjList,
    mut subgraph_unionfind: UnionFind<GraphNodeId>,
) -> (
    SecondaryMap<GraphNodeId, GraphSubgraphId>,
    SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
) {
    // We want the nodes of each subgraph to be listed in topo-sort order.
    // We could do this on each subgraph, or we could do it all at once on the
    // whole node graph by ignoring handoffs, which is what we do here:
    let topo_sort = graph_algorithms::topo_sort(
        nodes
            .iter()
            .filter(|&(_, node)| !matches!(node, Node::Handoff))
            .map(|(node_id, _)| node_id),
        |v| {
            preds
                .get(v)
                .into_iter()
                .flatten()
                .map(|(_src_idx, &(dst, _dst_idx))| dst)
                .filter(|&dst| !matches!(nodes[dst], Node::Handoff))
        },
    );

    let mut grouped_nodes: SecondaryMap<GraphNodeId, Vec<GraphNodeId>> = Default::default();
    for node_id in topo_sort {
        let repr_node = subgraph_unionfind.find(node_id);
        if !grouped_nodes.contains_key(repr_node) {
            grouped_nodes.insert(repr_node, Default::default());
        }
        grouped_nodes[repr_node].push(node_id);
    }

    let mut node_subgraph: SecondaryMap<GraphNodeId, GraphSubgraphId> = Default::default();
    let mut subgraph_nodes: SlotMap<GraphSubgraphId, Vec<GraphNodeId>> = Default::default();
    for (_repr_node, member_nodes) in grouped_nodes {
        subgraph_nodes.insert_with_key(|subgraph_id| {
            for &node_id in member_nodes.iter() {
                node_subgraph.insert(node_id, subgraph_id);
            }
            member_nodes
        });
    }
    (node_subgraph, subgraph_nodes)
}

fn find_subgraphs(
    nodes: &mut SlotMap<GraphNodeId, Node>,
    preds: &mut AdjList,
    succs: &mut AdjList,
    barrier_crossers: &[(GraphNodeId, GraphNodeId, IndexInt, DelayType)],
) -> (
    SecondaryMap<GraphNodeId, GraphSubgraphId>,
    SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
) {
    // Algorithm:
    // 1. Each node begins as its own subgraph.
    // 2. Collect edges. (Future optimization: sort so edges which should not be split across a handoff come first).
    // 3. For each edge, try to join `(to, from)` into the same subgraph.

    let (subgraph_unionfind, handoff_edges) =
        find_subgraph_unionfind(nodes, preds, succs, barrier_crossers);

    // Insert handoffs between subgraphs (or on subgraph self-loop edges)
    for edge in handoff_edges {
        let ((src, _src_idx), (dst, dst_idx)) = edge;

        // Already has a handoff, no need to insert one.
        if matches!(nodes[src], Node::Handoff) || matches!(nodes[dst], Node::Handoff) {
            continue;
        }

        insert_intermediate_node(nodes, preds, succs, Node::Handoff, (src, dst, dst_idx));
    }

    // Determine node's subgraph and subgraph's nodes.
    // This list of nodes in each subgraph are to be in topological sort order.
    // Eventually returned directly in the `PartitionedGraph`.
    let (node_subgraph, subgraph_nodes) = find_subgraph_collect(nodes, preds, subgraph_unionfind);
    (node_subgraph, subgraph_nodes)
}

/// Set `src` or `dst` color if `None` based on the other (if possible):
/// `None` indicates an op could be pull or push i.e. unary-in & unary-out.
/// So in that case we color `src` or `dst` based on its newfound neighbor (the other one).
///
/// Returns if `src` and `dst` can be in the same subgraph.
fn can_connect_colorize(
    node_color: &mut SecondaryMap<GraphNodeId, Option<Color>>,
    src: GraphNodeId,
    dst: GraphNodeId,
) -> bool {
    // Pull -> Pull
    // Push -> Push
    // Pull -> [Computation] -> Push
    // Push -> [Handoff] -> Pull
    let can_connect = match (node_color[src], node_color[dst]) {
        // Linear chain.
        (None, None) => true,

        // Infer left side.
        (None, Some(Color::Pull | Color::Comp)) => {
            node_color[src] = Some(Color::Pull);
            true
        }
        (None, Some(Color::Push | Color::Hoff)) => {
            node_color[src] = Some(Color::Push);
            true
        }

        // Infer right side.
        (Some(Color::Pull | Color::Hoff), None) => {
            node_color[dst] = Some(Color::Pull);
            true
        }
        (Some(Color::Comp | Color::Push), None) => {
            node_color[dst] = Some(Color::Push);
            true
        }

        // Both sides already specified.
        (Some(Color::Pull), Some(Color::Pull)) => true,
        (Some(Color::Pull), Some(Color::Comp)) => true,
        (Some(Color::Pull), Some(Color::Push)) => true,

        (Some(Color::Comp), Some(Color::Pull)) => false,
        (Some(Color::Comp), Some(Color::Comp)) => false,
        (Some(Color::Comp), Some(Color::Push)) => true,

        (Some(Color::Push), Some(Color::Pull)) => false,
        (Some(Color::Push), Some(Color::Comp)) => false,
        (Some(Color::Push), Some(Color::Push)) => true,

        // Handoffs are not part of subgraphs.
        (Some(Color::Hoff), Some(_)) => false,
        (Some(_), Some(Color::Hoff)) => false,
    };
    can_connect
}

fn find_subgraph_strata(
    nodes: &mut SlotMap<GraphNodeId, Node>,
    new_preds: &mut AdjList,
    new_succs: &mut AdjList,
    node_subgraph: &mut SecondaryMap<GraphNodeId, GraphSubgraphId>,
    subgraph_nodes: &mut SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
    barrier_crossers: &[(GraphNodeId, GraphNodeId, IndexInt, DelayType)],
) -> Result<SecondaryMap<GraphSubgraphId, usize>, ()> {
    // Determine subgraphs's stratum number.
    // Find SCCs ignoring `next_epoch()` edges, then do TopoSort on the resulting DAG.
    // (Cycles on cross-stratum negative edges are an error.)

    // Generate subgraph graph.
    let mut subgraph_preds: HashMap<GraphSubgraphId, Vec<GraphSubgraphId>> = Default::default();
    let mut subgraph_succs: HashMap<GraphSubgraphId, Vec<GraphSubgraphId>> = Default::default();

    for (node_id, node) in nodes.iter() {
        if matches!(node, Node::Handoff) {
            for &(pred, _) in new_preds[node_id].values() {
                let pred_sg = node_subgraph[pred];
                for &(succ, succ_idx) in new_succs[node_id].values() {
                    if barrier_crossers.contains(&(pred, succ, succ_idx, DelayType::Epoch)) {
                        continue;
                    }
                    let succ_sg = node_subgraph[succ];
                    subgraph_preds.entry(succ_sg).or_default().push(pred_sg);
                    subgraph_succs.entry(pred_sg).or_default().push(succ_sg);
                }
            }
        }
    }

    let scc = graph_algorithms::scc_kosaraju(
        subgraph_nodes.keys(),
        |v| subgraph_preds.get(&v).into_iter().flatten().cloned(),
        |u| subgraph_succs.get(&u).into_iter().flatten().cloned(),
    );

    let topo_sort_order = {
        // Condensed each SCC into a single node for toposort.
        let mut condensed_preds: HashMap<GraphSubgraphId, Vec<GraphSubgraphId>> =
            Default::default();
        for (u, preds) in subgraph_preds.iter() {
            condensed_preds
                .entry(scc[u])
                .or_default()
                .extend(preds.iter().map(|v| scc[v]));
        }

        graph_algorithms::topo_sort(subgraph_nodes.keys(), |v| {
            condensed_preds.get(&v).into_iter().flatten().cloned()
        })
    };

    // Negative (next stratum) connections between subgraphs. (Ignore `next_epoch()` connections).
    let subgraph_negative_connections: HashSet<_> = barrier_crossers
        .iter()
        .filter(|(_src, _dst, _dst_idx, delay_type)| DelayType::Stratum == *delay_type)
        .map(|(src, dst, _dst_idx, _delay_type)| (node_subgraph[*src], node_subgraph[*dst]))
        .collect();

    let mut subgraph_stratum: SecondaryMap<GraphSubgraphId, usize> =
        SecondaryMap::with_capacity(topo_sort_order.len());
    // Each subgraph stratum is the same as it's predacessors unless there is a negative edge.
    for sg_id in topo_sort_order {
        subgraph_stratum.insert(
            sg_id,
            subgraph_preds
                .get(&sg_id)
                .into_iter()
                .flatten()
                .filter_map(|&pred_sg_id| {
                    subgraph_stratum.get(pred_sg_id).map(|stratum| {
                        stratum
                            + (subgraph_negative_connections.contains(&(pred_sg_id, sg_id))
                                as usize)
                    })
                })
                .max()
                .unwrap_or(0),
        );
    }

    // Re-introduce the `next_epoch()` edges, ensuring they actually go to the next epoch.
    let max_stratum = subgraph_stratum.values().cloned().max().unwrap_or(0) + 1; // Used for `next_epoch()` delayer subgraphs.
    for &(src, dst, dst_idx, input_barrier) in barrier_crossers.iter() {
        let src_sg = node_subgraph[src];
        let dst_sg = node_subgraph[dst];
        let src_stratum = subgraph_stratum[src_sg];
        let dst_stratum = subgraph_stratum[dst_sg];
        match input_barrier {
            DelayType::Epoch => {
                // If epoch edge goes foreward in stratum, need to buffer.
                // (TODO(mingwei): could use a different kind of handoff.)
                if src_stratum <= dst_stratum {
                    // We inject a new subgraph between the src/dst which runs as the last stratum
                    // of the epoch and therefore delays the data until the next epoch.

                    // Before: A (src) -> H -> B (dst)
                    let (hoff_node_id, _hoff_idx) = new_preds[dst][&dst_idx];
                    let new_node_id = insert_intermediate_node(
                        nodes,
                        new_preds,
                        new_succs,
                        Node::Operator(parse_quote! { identity() }),
                        (hoff_node_id, dst, dst_idx),
                    );
                    let new_subgraph_id = subgraph_nodes.insert(vec![new_node_id]);
                    subgraph_stratum.insert(new_subgraph_id, max_stratum);
                    node_subgraph.insert(new_node_id, new_subgraph_id);
                    // Intermediate: A (src) -> H -> X -> B (dst)
                    let _hoff_node_id = insert_intermediate_node(
                        nodes,
                        new_preds,
                        new_succs,
                        Node::Handoff,
                        (new_node_id, dst, dst_idx),
                    );
                    // After: A (src) -> H -> X -> H' -> B (dst)
                }
            }
            DelayType::Stratum => {
                // Any negative edges which go onto the same or previous stratum are bad.
                // Indicates an unbroken negative cycle.
                if dst_stratum <= src_stratum {
                    dst_idx
                        .span()
                        .unwrap()
                        .error("Negative edge creates a negative cycle which must be broken with a `next_epoch()` operator.")
                        .emit();
                    return Err(());
                }
            }
        }
    }

    Ok(subgraph_stratum)
}

// Find the input (recv) and output (send) handoffs for each subgraph.
fn find_subgraph_handoffs(
    nodes: &SlotMap<GraphNodeId, Node>,
    new_succs: &AdjList,
    node_subgraph: &SecondaryMap<GraphNodeId, GraphSubgraphId>,
    subgraph_nodes: &SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
) -> (
    SecondaryMap<GraphSubgraphId, Vec<GraphNodeId>>,
    SecondaryMap<GraphSubgraphId, Vec<GraphNodeId>>,
) {
    // Get data on handoff src and dst subgraphs.
    let mut subgraph_recv_handoffs: SecondaryMap<GraphSubgraphId, Vec<GraphNodeId>> =
        subgraph_nodes
            .keys()
            .map(|k| (k, Default::default()))
            .collect();
    let mut subgraph_send_handoffs = subgraph_recv_handoffs.clone();

    // For each edge in the graph, if `src` or `dst` are a handoff then assign
    // that handoff the to neighboring subgraphs (the other of `src`/`dst`).
    // (Mingwei: alternatively, could iterate nodes instead and just look at pred/succ).
    for edge in iter_edges(new_succs) {
        let ((src, _), (dst, _)) = edge;
        let (src_node, dst_node) = (&nodes[src], &nodes[dst]);
        match (src_node, dst_node) {
            (Node::Operator(_), Node::Operator(_)) => {}
            (Node::Operator(_), Node::Handoff) => {
                subgraph_send_handoffs[node_subgraph[src]].push(dst);
            }
            (Node::Handoff, Node::Operator(_)) => {
                subgraph_recv_handoffs[node_subgraph[dst]].push(src);
            }
            (Node::Handoff, Node::Handoff) => {
                Span::call_site().unwrap().error(format!(
                    "Internal Error: Consecutive handoffs {:?} -> {:?}",
                    src.data(),
                    dst.data()
                ));
            }
        }
    }

    (subgraph_recv_handoffs, subgraph_send_handoffs)
}

impl TryFrom<FlatGraph> for PartitionedGraph {
    type Error = (); // TODO(mingwei).

    fn try_from(flat_graph: FlatGraph) -> Result<Self, Self::Error> {
        let FlatGraph {
            mut nodes,
            mut preds,
            mut succs,
            ..
        } = flat_graph;

        // Pairs of node IDs which cross stratums or epochs and therefore cannot be in the same subgraph.
        let barrier_crossers = find_barrier_crossers(&nodes, &succs);

        let (mut node_subgraph, mut subgraph_nodes) =
            find_subgraphs(&mut nodes, &mut preds, &mut succs, &barrier_crossers);

        let subgraph_stratum = find_subgraph_strata(
            &mut nodes,
            &mut preds,
            &mut succs,
            &mut node_subgraph,
            &mut subgraph_nodes,
            &barrier_crossers,
        )?;

        let (subgraph_recv_handoffs, subgraph_send_handoffs) =
            find_subgraph_handoffs(&nodes, &succs, &node_subgraph, &subgraph_nodes);

        Ok(PartitionedGraph {
            nodes,
            preds,
            succs,
            node_subgraph,

            subgraph_nodes,
            subgraph_stratum,
            subgraph_recv_handoffs,
            subgraph_send_handoffs,
        })
    }
}

/// `edge`: (src, dst, dst_idx)
///
/// Before: A (src) ------------> B (dst)
/// After:  A (src) -> X (new) -> B (dst)
fn insert_intermediate_node(
    nodes: &mut SlotMap<GraphNodeId, Node>,
    preds: &mut AdjList,
    succs: &mut AdjList,
    node: Node,
    edge: (GraphNodeId, GraphNodeId, IndexInt),
) -> GraphNodeId {
    let ii0 = IndexInt {
        value: 0,
        span: Span::call_site(),
    };

    let (src, dst, dst_idx) = edge;
    let new_id = nodes.insert(node);

    // X <- B
    let (src_alt, src_idx) = preds[dst]
        .insert(dst_idx, (new_id, ii0))
        .expect("Pred edge should exist.");
    assert_eq!(src, src_alt, "Src should match.");

    // A -> X
    let (dst_alt, dst_idx_alt) = succs[src]
        .insert(src_idx, (new_id, ii0))
        .expect("Succ edge should exist.");
    assert_eq!(dst, dst_alt, "Dst should match.");
    assert_eq!(dst_idx, dst_idx_alt, "Dst idx should match.");

    // A <- X
    preds.insert(new_id, [(ii0, (src, src_idx))].into());
    // X -> B
    succs.insert(new_id, [(ii0, (dst, dst_idx))].into());

    new_id
}
