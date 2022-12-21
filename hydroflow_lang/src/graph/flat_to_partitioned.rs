use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Span;
use slotmap::{Key, SecondaryMap, SlotMap};
use syn::parse_quote;
use syn::spanned::Spanned;

use crate::diagnostic::{Diagnostic, Level};
use crate::union_find::UnionFind;

use super::di_mul_graph::DiMulGraph;
use super::flat_graph::FlatGraph;
use super::ops::{DelayType, OPERATORS};
use super::partitioned_graph::PartitionedGraph;
use super::{
    graph_algorithms, node_color, Color, GraphEdgeId, GraphNodeId, GraphSubgraphId, Node,
    PortIndexValue,
};

fn find_barrier_crossers(
    nodes: &SlotMap<GraphNodeId, Node>,
    ports: &SecondaryMap<GraphEdgeId, (PortIndexValue, PortIndexValue)>,
    graph: &DiMulGraph<GraphNodeId, GraphEdgeId>,
) -> SecondaryMap<GraphEdgeId, DelayType> {
    graph
        .edges()
        .filter_map(|(edge_id, (_src, dst))| {
            let (_src_idx, dst_idx) = &ports[edge_id];
            if let Node::Operator(dst_operator) = &nodes[dst] {
                let dst_name = &*dst_operator.name_string();
                OPERATORS
                    .iter()
                    .find(|&op| dst_name == op.name)
                    .and_then(|op_constraints| (op_constraints.input_delaytype_fn)(dst_idx))
                    .map(|input_barrier| (edge_id, input_barrier))
            } else {
                None
            }
        })
        .collect()
}

fn find_subgraph_unionfind(
    nodes: &SlotMap<GraphNodeId, Node>,
    graph: &DiMulGraph<GraphNodeId, GraphEdgeId>,
    barrier_crossers: &SecondaryMap<GraphEdgeId, DelayType>,
) -> (UnionFind<GraphNodeId>, BTreeSet<GraphEdgeId>) {
    // Pre-calculate node colors.
    let mut node_color: SecondaryMap<GraphNodeId, Option<Color>> = nodes
        .keys()
        .map(|node_id| {
            let inn_degree = graph.degree_in(node_id);
            let out_degree = graph.degree_out(node_id);
            let op_color = node_color(&nodes[node_id], inn_degree, out_degree);
            (node_id, op_color)
        })
        .collect();

    let mut subgraph_unionfind: UnionFind<GraphNodeId> = UnionFind::with_capacity(nodes.len());
    // Will contain all edges which are handoffs. Starts out with all edges and
    // we remove from this set as we construct subgraphs.
    let mut handoff_edges: BTreeSet<GraphEdgeId> =
        graph.edges().map(|(edge_id, _)| edge_id).collect();
    // Would sort edges here for priority (for now, no sort/priority).

    // Each edge gets looked at in order. However we may not know if a linear
    // chain of operators is PUSH vs PULL until we look at the ends. A fancier
    // algorithm would know to handle linear chains from the outside inward.
    // But instead we just run through the edges in a loop until no more
    // progress is made. Could have some sort of O(N^2) pathological worst
    // case.
    let mut progress = true;
    while progress {
        progress = false;
        for (edge_id, (src, dst)) in graph.edges() {
            // Ignore (1) already added edges as well as (2) new self-cycles.
            if subgraph_unionfind.same_set(src, dst) {
                // Note this might be triggered even if the edge (src, dst) is not in the subgraph (not case 1).
                // This prevents self-loops which would violate the in-out tree structure (case 2).
                // Handoffs will be inserted later for this self-loop.
                continue;
            }

            // Ignore if would join stratum crossers (next edges).
            if barrier_crossers.iter().any(|(edge_id, _)| {
                let (x_src, x_dst) = graph.edge(edge_id).unwrap();
                (subgraph_unionfind.same_set(x_src, src) && subgraph_unionfind.same_set(x_dst, dst))
                    || (subgraph_unionfind.same_set(x_src, dst)
                        && subgraph_unionfind.same_set(x_dst, src))
            }) {
                continue;
            }

            if can_connect_colorize(&mut node_color, src, dst) {
                // At this point we have selected this edge and its src & dst to be
                // within a single subgraph.
                subgraph_unionfind.union(src, dst);
                assert!(handoff_edges.remove(&edge_id));
                progress = true;
            }
        }
    }

    (subgraph_unionfind, handoff_edges)
}

/// Builds the datastructures for checking which subgraph each node belongs to
/// after handoffs have already been inserted to partition subgraphs.
/// This list of nodes in each subgraph are returned in topological sort order.
fn make_subgraph_collect(
    nodes: &SlotMap<GraphNodeId, Node>,
    graph: &DiMulGraph<GraphNodeId, GraphEdgeId>,
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
            .filter(|&(_, node)| !matches!(node, Node::Handoff { .. }))
            .map(|(node_id, _)| node_id),
        |v| {
            graph
                .predecessor_nodes(v)
                .filter(|&pred| !matches!(nodes[pred], Node::Handoff { .. }))
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

/// Find subgraph and insert handoffs.
/// Modifies barrier_crossers so that the edge OUT of an inserted handoff has
/// the DelayType data.
fn make_subgraphs(
    nodes: &mut SlotMap<GraphNodeId, Node>,
    ports: &mut SecondaryMap<GraphEdgeId, (PortIndexValue, PortIndexValue)>,
    graph: &mut DiMulGraph<GraphNodeId, GraphEdgeId>,
    barrier_crossers: &mut SecondaryMap<GraphEdgeId, DelayType>,
) -> (
    SecondaryMap<GraphNodeId, GraphSubgraphId>,
    SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
) {
    // Algorithm:
    // 1. Each node begins as its own subgraph.
    // 2. Collect edges. (Future optimization: sort so edges which should not be split across a handoff come first).
    // 3. For each edge, try to join `(to, from)` into the same subgraph.

    graph.assert_valid();

    let (subgraph_unionfind, handoff_edges) =
        find_subgraph_unionfind(nodes, graph, barrier_crossers);

    // Insert handoffs between subgraphs (or on subgraph self-loop edges)
    for edge_id in handoff_edges {
        let (src_id, dst_id) = graph.edge(edge_id).unwrap();

        // Already has a handoff, no need to insert one.
        let src_node = &nodes[src_id];
        let dst_node = &nodes[dst_id];
        if matches!(src_node, Node::Handoff { .. }) || matches!(dst_node, Node::Handoff { .. }) {
            continue;
        }

        let hoff = Node::Handoff {
            src_span: src_node.span(),
            dst_span: dst_node.span(),
        };
        let (_node_id, out_edge_id) = insert_intermediate_node(nodes, ports, graph, hoff, edge_id);

        // Update barrier_crossers for inserted node.
        if let Some(delay_type) = barrier_crossers.remove(edge_id) {
            barrier_crossers.insert(out_edge_id, delay_type);
        }
    }

    // Determine node's subgraph and subgraph's nodes.
    // This list of nodes in each subgraph are to be in topological sort order.
    // Eventually returned directly in the `PartitionedGraph`.
    let (node_subgraph, subgraph_nodes) = make_subgraph_collect(nodes, graph, subgraph_unionfind);
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
        // Linear chain, can't connect because it may cause future conflicts.
        // But if it doesn't in the _future_ we can connect it (once either/both ends are determined).
        (None, None) => false,

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
    ports: &mut SecondaryMap<GraphEdgeId, (PortIndexValue, PortIndexValue)>,
    graph: &mut DiMulGraph<GraphNodeId, GraphEdgeId>,
    node_subgraph: &mut SecondaryMap<GraphNodeId, GraphSubgraphId>,
    subgraph_nodes: &mut SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
    barrier_crossers: &SecondaryMap<GraphEdgeId, DelayType>,
) -> Result<SecondaryMap<GraphSubgraphId, usize>, Diagnostic> {
    // Determine subgraphs's stratum number.
    // Find SCCs ignoring `next_tick()` edges, then do TopoSort on the resulting DAG.
    // (Cycles on cross-stratum negative edges are an error.)

    // Generate a subgraph graph. I.e. each node is a subgraph.
    // Edges are connections between subgraphs, ignoring tick-crossers.
    // TODO: use DiMulGraph here?
    let mut subgraph_preds: BTreeMap<GraphSubgraphId, Vec<GraphSubgraphId>> = Default::default();
    let mut subgraph_succs: BTreeMap<GraphSubgraphId, Vec<GraphSubgraphId>> = Default::default();

    // Negative (next stratum) connections between subgraphs. (Ignore `next_tick()` connections).
    let mut subgraph_negative_connections: BTreeSet<(GraphSubgraphId, GraphSubgraphId)> =
        Default::default();

    for (node_id, node) in nodes.iter() {
        if matches!(node, Node::Handoff { .. }) {
            assert_eq!(1, graph.successors(node_id).count());
            let (succ_edge, succ) = graph.successors(node_id).next().unwrap();

            // Ignore tick edges.
            if Some(&DelayType::Tick) == barrier_crossers.get(succ_edge) {
                continue;
            }

            assert_eq!(1, graph.predecessor_nodes(node_id).count());
            let pred = graph.predecessor_nodes(node_id).next().unwrap();

            let pred_sg = node_subgraph[pred];
            let succ_sg = node_subgraph[succ];

            subgraph_preds.entry(succ_sg).or_default().push(pred_sg);
            subgraph_succs.entry(pred_sg).or_default().push(succ_sg);

            if Some(&DelayType::Stratum) == barrier_crossers.get(succ_edge) {
                subgraph_negative_connections.insert((pred_sg, succ_sg));
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
        let mut condensed_preds: BTreeMap<GraphSubgraphId, Vec<GraphSubgraphId>> =
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

    let mut subgraph_stratum: SecondaryMap<GraphSubgraphId, usize> =
        SecondaryMap::with_capacity(topo_sort_order.len());
    // Each subgraph stratum is the same as it's predecessors unless there is a negative edge.
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

    // Re-introduce the `next_tick()` edges, ensuring they actually go to the next tick.
    let max_stratum = subgraph_stratum.values().cloned().max().unwrap_or(0) + 1; // Used for `next_tick()` delayer subgraphs.
    for (edge_id, &delay_type) in barrier_crossers.iter() {
        let (hoff, dst) = graph.edge(edge_id).unwrap();
        assert_eq!(1, graph.predecessor_nodes(hoff).count());
        let src = graph.predecessor_nodes(hoff).next().unwrap();

        let src_sg = node_subgraph[src];
        let dst_sg = node_subgraph[dst];
        let src_stratum = subgraph_stratum[src_sg];
        let dst_stratum = subgraph_stratum[dst_sg];
        match delay_type {
            DelayType::Tick => {
                // If tick edge goes foreward in stratum, need to buffer.
                // (TODO(mingwei): could use a different kind of handoff.)
                if src_stratum <= dst_stratum {
                    // We inject a new subgraph between the src/dst which runs as the last stratum
                    // of the tick and therefore delays the data until the next tick.

                    // Before: A (src) -> H -> B (dst)
                    // Then add intermediate identity:
                    let (new_node_id, new_edge_id) = insert_intermediate_node(
                        nodes,
                        ports,
                        graph,
                        // TODO(mingwei): Proper span w/ `parse_quote_spanned!`?
                        Node::Operator(parse_quote! { identity() }),
                        edge_id,
                    );
                    // Intermediate: A (src) -> H -> ID -> B (dst)
                    let hoff = Node::Handoff {
                        src_span: Span::call_site(), // TODO(mingwei): Proper spanning?
                        dst_span: Span::call_site(),
                    };
                    let (_hoff_node_id, _hoff_edge_id) =
                        insert_intermediate_node(nodes, ports, graph, hoff, new_edge_id);
                    // After: A (src) -> H -> ID -> H' -> B (dst)

                    // Set stratum numbers.
                    let new_subgraph_id = subgraph_nodes.insert(vec![new_node_id]);
                    subgraph_stratum.insert(new_subgraph_id, max_stratum);
                    node_subgraph.insert(new_node_id, new_subgraph_id);
                }
            }
            DelayType::Stratum => {
                // Any negative edges which go onto the same or previous stratum are bad.
                // Indicates an unbroken negative cycle.
                if dst_stratum <= src_stratum {
                    let (_src_idx, dst_idx) = &ports[edge_id];
                    return Err(Diagnostic::spanned(dst_idx.span(), Level::Error, "Negative edge creates a negative cycle which must be broken with a `next_tick()` operator."));
                }
            }
        }
    }

    Ok(subgraph_stratum)
}

// Find the input (recv) and output (send) handoffs for each subgraph.
fn find_subgraph_handoffs(
    nodes: &SlotMap<GraphNodeId, Node>,
    graph: &DiMulGraph<GraphNodeId, GraphEdgeId>,
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
    for (_edge_id, (src, dst)) in graph.edges() {
        let (src_node, dst_node) = (&nodes[src], &nodes[dst]);
        match (src_node, dst_node) {
            (Node::Operator(_), Node::Operator(_)) => {}
            (Node::Operator(_), Node::Handoff { .. }) => {
                subgraph_send_handoffs[node_subgraph[src]].push(dst);
            }
            (Node::Handoff { .. }, Node::Operator(_)) => {
                subgraph_recv_handoffs[node_subgraph[dst]].push(src);
            }
            (Node::Handoff { .. }, Node::Handoff { .. }) => {
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
    type Error = Diagnostic;

    fn try_from(flat_graph: FlatGraph) -> Result<Self, Self::Error> {
        let FlatGraph {
            mut nodes,
            mut graph,
            mut ports,
            ..
        } = flat_graph;

        // Pairs of node IDs which cross stratums or ticks and therefore cannot be in the same subgraph.
        let mut barrier_crossers = find_barrier_crossers(&nodes, &ports, &graph);

        let (mut node_subgraph, mut subgraph_nodes) =
            make_subgraphs(&mut nodes, &mut ports, &mut graph, &mut barrier_crossers);

        let subgraph_stratum = find_subgraph_strata(
            &mut nodes,
            &mut ports,
            &mut graph,
            &mut node_subgraph,
            &mut subgraph_nodes,
            &barrier_crossers,
        )?;

        let (subgraph_recv_handoffs, subgraph_send_handoffs) =
            find_subgraph_handoffs(&nodes, &graph, &node_subgraph, &subgraph_nodes);

        Ok(PartitionedGraph {
            nodes,
            graph,
            ports,
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
///
/// Returns the ID of X & ID of edge OUT of X.
fn insert_intermediate_node(
    nodes: &mut SlotMap<GraphNodeId, Node>,
    ports: &mut SecondaryMap<GraphEdgeId, (PortIndexValue, PortIndexValue)>,
    graph: &mut DiMulGraph<GraphNodeId, GraphEdgeId>,
    node: Node,
    edge_id: GraphEdgeId,
) -> (GraphNodeId, GraphEdgeId) {
    let span = node.span();
    let node_id = nodes.insert(node);
    let (e0, e1) = graph.insert_intermediate_node(node_id, edge_id).unwrap();

    let (src_idx, dst_idx) = ports.remove(edge_id).unwrap();
    ports.insert(e0, (src_idx, PortIndexValue::Elided(span)));
    ports.insert(e1, (PortIndexValue::Elided(span), dst_idx));

    (node_id, e1)
}
