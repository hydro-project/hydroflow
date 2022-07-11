//! Graph representation stages for Hydroflow graphs.

use std::collections::{HashMap, HashSet};

use proc_macro2::Span;
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use syn::spanned::Spanned;
use syn::LitInt;

use crate::parse::Operator;
use crate::pretty_span::PrettySpan;
use crate::union_find::UnionFind;

use self::flat_graph::FlatGraph;
use self::partitioned_graph::PartitionedGraph;

pub mod flat_graph;
pub mod ops;
pub mod partitioned_graph;

new_key_type! {
    /// ID to identify a node (operator or handoff) in both [`flat_graph::FlatGraph`]
    /// and [`partitioned_graph::PartitionedGraph`].
    pub struct NodeId;
}
new_key_type! {
    /// ID to identify a subgraph in [`partitioned_graph::PartitionedGraph`].
    pub struct SubgraphId;
}

pub type EdgePort = (NodeId, LitInt);
pub type EdgePortRef<'a> = (NodeId, &'a LitInt);

pub enum Node {
    Operator(Operator),
    Handoff,
}
impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Operator(operator) => {
                write!(f, "Node::Operator({} span)", PrettySpan(operator.span()))
            }
            Self::Handoff => write!(f, "Node::Handoff"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    /// Pull (green)
    Pull,
    /// Push (blue)
    Push,
    /// Computation (yellow)
    Comp,
    /// Handoff (red) -- not a color for operators, inserted between subgraphs.
    Hoff,
}

pub fn node_color(node: &Node, inn_degree: usize, out_degree: usize) -> Option<Color> {
    // Determine op color based on in and out degree. If linear (1 in 1 out), color is None.
    match node {
        Node::Operator(_) => match (1 < inn_degree, 1 < out_degree) {
            (true, true) => Some(Color::Comp),
            (true, false) => Some(Color::Pull),
            (false, true) => Some(Color::Push),
            (false, false) => match (inn_degree, out_degree) {
                (0, _) => Some(Color::Pull),
                (_, 0) => Some(Color::Push),
                _same => None,
            },
        },
        Node::Handoff => Some(Color::Hoff),
    }
}

impl From<FlatGraph> for PartitionedGraph {
    fn from(mut flat_graph: FlatGraph) -> Self {
        // Algorithm:
        // 1. Each node begins as its own subgraph.
        // 2. Collect edges. Sort so edges which should not be split across a handoff come first.
        // 3. For each edge, try to join `(to, from)` into the same subgraph.

        let mut node_color: SecondaryMap<NodeId, Option<Color>> = flat_graph
            .nodes
            .keys()
            .map(|node_id| {
                let inn_degree = flat_graph.preds[node_id].len();
                let out_degree = flat_graph.succs[node_id].len();
                let op_color = node_color(&flat_graph.nodes[node_id], inn_degree, out_degree);
                (node_id, op_color)
            })
            .collect();
        let mut node_union: UnionFind<NodeId> = UnionFind::with_capacity(flat_graph.nodes.len());
        // All edges which belong to a single subgraph. Other & self-edges become handoffs.
        let mut subgraph_edges: HashSet<(EdgePortRef, EdgePortRef)> = Default::default();

        // Sort edges here (for now, no sort/priority).
        loop {
            let mut updated = false;
            for ((src, src_idx), (dst, dst_idx)) in iter_edges(&flat_graph.succs) {
                if node_union.same_set(src, dst) {
                    // Note this might be triggered even if the edge (src, dst) is not in the subgraph.
                    // This prevents self-loops. Handoffs needed to break self loops.
                    continue;
                }

                // Set `src` or `dst` color if `None` based on the other (if possible):
                // Pull -> Pull
                // Push -> Push
                // Pull -> [Comp] -> Push
                // Push -> [Hoff] -> Pull
                match (node_color[src], node_color[dst]) {
                    (Some(_), Some(_)) => (),
                    (None, None) => (),
                    (None, Some(dst_color)) => {
                        node_color[src] = Some(match dst_color {
                            Color::Comp => Color::Pull,
                            Color::Hoff => Color::Push,
                            pull_or_push => pull_or_push,
                        });
                        updated = true;
                    }
                    (Some(src_color), None) => {
                        node_color[dst] = Some(match src_color {
                            Color::Comp => Color::Push,
                            Color::Hoff => Color::Pull,
                            pull_or_push => pull_or_push,
                        });
                        updated = true;
                    }
                }

                // If SRC and DST can be in the same subgraph.
                let can_connect = match (node_color[src], node_color[dst]) {
                    (Some(Color::Pull), Some(Color::Pull)) => true,
                    (Some(Color::Pull), Some(Color::Comp)) => true,
                    (Some(Color::Pull), Some(Color::Push)) => true,

                    (Some(Color::Comp | Color::Push), Some(Color::Pull)) => false,
                    (Some(Color::Comp | Color::Push), Some(Color::Comp)) => false,
                    (Some(Color::Comp | Color::Push), Some(Color::Push)) => true,

                    // Handoffs are not part of subgraphs.
                    (Some(Color::Hoff), Some(_)) => false,
                    (Some(_), Some(Color::Hoff)) => false,

                    // Linear chain.
                    (None, None) => true,

                    _some_none => unreachable!(),
                };
                if can_connect {
                    node_union.union(src, dst);
                    subgraph_edges.insert(((src, src_idx), (dst, dst_idx)));
                    updated = true;
                }
            }
            if !updated {
                break;
            }
        }

        let mut new_preds: SecondaryMap<NodeId, HashMap<LitInt, EdgePort>> = flat_graph
            .nodes
            .keys()
            .map(|k| (k, Default::default()))
            .collect();
        let mut new_succs: SecondaryMap<NodeId, HashMap<LitInt, EdgePort>> = flat_graph
            .nodes
            .keys()
            .map(|k| (k, Default::default()))
            .collect();

        // Copy over edges, inserting handoffs between subgraphs (or on subgraph self-edges) when needed.
        for edge in iter_edges(&flat_graph.succs) {
            let is_subgraph_edge = subgraph_edges.contains(&edge); // Internal subgraph edges are not handoffs.
            let ((src, src_idx), (dst, dst_idx)) = edge;

            // Already has a handoff, no need to insert one.
            if is_subgraph_edge
                || matches!(flat_graph.nodes[src], Node::Handoff)
                || matches!(flat_graph.nodes[dst], Node::Handoff)
            {
                new_preds[dst].insert(dst_idx.clone(), (src, src_idx.clone()));
                new_succs[src].insert(src_idx.clone(), (dst, dst_idx.clone()));
            } else {
                // Needs handoff inserted.
                // A -> H -> Z
                let hoff_id = flat_graph.nodes.insert(Node::Handoff);
                new_preds.insert(hoff_id, Default::default());
                new_succs.insert(hoff_id, Default::default());

                // A -> H.
                new_succs[src].insert(
                    src_idx.clone(),
                    (hoff_id, LitInt::new("0", Span::call_site())),
                );
                // A <- H.
                new_preds[hoff_id]
                    .insert(LitInt::new("0", Span::call_site()), (src, src_idx.clone()));
                // H <- Z.
                new_preds[dst].insert(
                    dst_idx.clone(),
                    (hoff_id, LitInt::new("0", Span::call_site())),
                );
                // H -> Z.
                new_succs[hoff_id]
                    .insert(LitInt::new("0", Span::call_site()), (dst, dst_idx.clone()));
            }
        }

        // // Mapping from representative `NodeId` to the `SubgraphId`.
        // let mut node_to_subgraph = HashMap::new();
        // // List of nodes in each `SubgraphId`.
        // let mut subgraph_nodes: SlotMap<SubgraphId, Vec<NodeId>> =
        //     SlotMap::with_capacity_and_key(flat_graph.nodes.len());
        // // `SubgraphId` for each `NodeId`.
        // let node_subgraph = flat_graph
        //     .nodes
        //     .iter()
        //     .filter_map(|(node_id, node)| match node {
        //         Node::Operator(_op) => {
        //             let repr_id = node_union.find(node_id);
        //             let subgraph_id = *node_to_subgraph
        //                 .entry(repr_id)
        //                 .or_insert_with(|| subgraph_nodes.insert(Default::default()));
        //             subgraph_nodes[subgraph_id].push(node_id);
        //             Some((node_id, subgraph_id))
        //         }
        //         Node::Handoff => None,
        //     })
        //     .collect();

        // Determine node's subgraph and subgraph's nodes in topological sort order.
        let (node_subgraph, subgraph_nodes) = {
            struct SubgraphTopoSort<'a> {
                nodes: &'a SlotMap<NodeId, Node>,
                preds: &'a SecondaryMap<NodeId, HashMap<LitInt, (NodeId, LitInt)>>,
                node_union: &'a mut UnionFind<NodeId>,
                marked: HashSet<NodeId>,
                subgraph_nodes: SecondaryMap<NodeId, Vec<NodeId>>,
            }
            impl<'a> SubgraphTopoSort<'a> {
                pub fn visit(&mut self, node_id: NodeId) {
                    // Already marked.
                    if self.marked.contains(&node_id) {
                        return;
                    }
                    // Ignore handoff nodes.
                    if matches!(self.nodes[node_id], Node::Handoff) {
                        return;
                    }

                    for &(next_back, _) in self.preds[node_id].values() {
                        if self.node_union.same_set(node_id, next_back) {
                            self.visit(next_back);
                        }
                    }

                    let repr_node = self.node_union.find(node_id);
                    if !self.subgraph_nodes.contains_key(repr_node) {
                        self.subgraph_nodes.insert(repr_node, Default::default());
                    }
                    self.subgraph_nodes[repr_node].push(node_id);
                    self.marked.insert(node_id);
                }
            }

            let (marked, subgraph_nodes) = Default::default();
            let mut sg_topo_sort = SubgraphTopoSort {
                nodes: &flat_graph.nodes,
                preds: &new_preds,
                node_union: &mut node_union,
                marked,
                subgraph_nodes,
            };
            for node_id in flat_graph.nodes.keys() {
                sg_topo_sort.visit(node_id);
            }

            let mut node_subgraph: SecondaryMap<NodeId, SubgraphId> = Default::default();
            let mut subgraph_nodes: SlotMap<SubgraphId, Vec<NodeId>> = Default::default();
            for (_repr_node, member_nodes) in sg_topo_sort.subgraph_nodes {
                subgraph_nodes.insert_with_key(|subgraph_id| {
                    for &node_id in member_nodes.iter() {
                        node_subgraph.insert(node_id, subgraph_id);
                    }
                    member_nodes
                });
            }

            (node_subgraph, subgraph_nodes)
        };

        PartitionedGraph {
            nodes: flat_graph.nodes,
            preds: new_preds,
            succs: new_succs,
            node_subgraph,
            subgraph_nodes,
        }
    }
}

pub(crate) fn iter_edges(
    succs: &SecondaryMap<NodeId, HashMap<LitInt, (NodeId, LitInt)>>,
) -> impl '_ + Iterator<Item = (EdgePortRef, EdgePortRef)> {
    succs.iter().flat_map(|(src, succs)| {
        succs
            .iter()
            .map(move |(src_idx, (dst, dst_idx))| ((src, src_idx), (*dst, dst_idx)))
    })
}
