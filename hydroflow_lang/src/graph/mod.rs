//! Graph representation stages for Hydroflow graphs.

use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::Hash;

use proc_macro2::Span;
use slotmap::{new_key_type, Key, SecondaryMap, SlotMap};
use syn::parse_quote;
use syn::spanned::Spanned;

use crate::graph::ops::OPERATORS;
use crate::parse::{IndexInt, Operator};
use crate::pretty_span::PrettySpan;
use crate::union_find::UnionFind;

use self::flat_graph::FlatGraph;
use self::ops::InputBarrier;
use self::partitioned_graph::PartitionedGraph;

pub mod flat_graph;
pub mod ops;
pub mod partitioned_graph;
pub mod serde_graph;

new_key_type! {
    /// ID to identify a node (operator or handoff) in both [`flat_graph::FlatGraph`]
    /// and [`partitioned_graph::PartitionedGraph`].
    pub struct GraphNodeId;
}
new_key_type! {
    /// ID to identify a subgraph in [`partitioned_graph::PartitionedGraph`].
    pub struct GraphSubgraphId;
}

pub type EdgePort = (GraphNodeId, IndexInt);
pub type EdgePortRef<'a> = (GraphNodeId, &'a IndexInt);
/// BTreeMap is used to ensure iteration order matches `IndexInt` order.
pub type OutboundEdges = BTreeMap<IndexInt, EdgePort>;

pub enum Node {
    Operator(Operator),
    Handoff,
}
impl Spanned for Node {
    fn span(&self) -> Span {
        match self {
            Node::Operator(op) => op.span(),
            Node::Handoff => Span::call_site(),
        }
    }
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

struct TopoSort<Id, PredsFn>
where
    Id: Hash + Eq,
{
    preds_fn: PredsFn,
    marked: HashSet<Id>,
    order: Vec<Id>,
}
impl<Id, PredsFn, PredsIter> TopoSort<Id, PredsFn>
where
    Id: Copy + Hash + Eq,
    PredsFn: FnMut(Id) -> PredsIter,
    PredsIter: IntoIterator<Item = Id>,
{
    pub fn visit(&mut self, node_id: Id) {
        // Already marked (cycle).
        if self.marked.contains(&node_id) {
            return;
        }

        self.marked.insert(node_id);
        for next_back in (self.preds_fn)(node_id) {
            self.visit(next_back);
        }
        self.order.push(node_id);
    }
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

type AdjList = SecondaryMap<GraphNodeId, BTreeMap<IndexInt, (GraphNodeId, IndexInt)>>;

fn find_subgraphs(
    nodes: &mut SlotMap<GraphNodeId, Node>,
    preds: &mut AdjList,
    succs: &mut AdjList,
    barrier_crossers: &Vec<(GraphNodeId, GraphNodeId, IndexInt, InputBarrier)>,
) -> (
    AdjList,
    AdjList,
    SecondaryMap<GraphNodeId, GraphSubgraphId>,
    SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
) {
    // Algorithm:
    // 1. Each node begins as its own subgraph.
    // 2. Collect edges. (Future optimization: sort so edges which should not be split across a handoff come first).
    // 3. For each edge, try to join `(to, from)` into the same subgraph.

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
    let mut node_union: UnionFind<GraphNodeId> = UnionFind::with_capacity(nodes.len());
    // All edges which belong to a single subgraph. Other & self-edges become handoffs.
    let mut subgraph_edges: HashSet<(EdgePortRef, EdgePortRef)> = Default::default();

    // Sort edges here (for now, no sort/priority).
    loop {
        let mut updated = false;
        for ((src, src_idx), (dst, dst_idx)) in iter_edges(&succs) {
            // Ignore new self-loops.
            if node_union.same_set(src, dst) {
                // Note this might be triggered even if the edge (src, dst) is not in the subgraph.
                // This prevents self-loops. Handoffs needed to break self loops.
                continue;
            }

            // Ignore if would join stratum crossers.
            if barrier_crossers
                .iter()
                .any(|&(x_src, x_dst, _x_dst_idx, _x_input_barrier)| {
                    (node_union.same_set(x_src, src) && node_union.same_set(x_dst, dst))
                        || (node_union.same_set(x_src, dst) && node_union.same_set(x_dst, src))
                })
            {
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

    // Copy of `self.preds` for the output.
    let mut new_preds: SecondaryMap<GraphNodeId, OutboundEdges> =
        nodes.keys().map(|k| (k, Default::default())).collect();
    // Copy of `self.succs` for the output.
    let mut new_succs: SecondaryMap<GraphNodeId, OutboundEdges> =
        nodes.keys().map(|k| (k, Default::default())).collect();

    // Copy over edges, inserting handoffs between subgraphs (or on subgraph self-edges) when needed.
    for edge in iter_edges(&succs) {
        let is_subgraph_edge = subgraph_edges.contains(&edge); // Internal subgraph edges are not handoffs.
        let ((src, src_idx), (dst, dst_idx)) = edge;

        // Already has a handoff, no need to insert one.
        if is_subgraph_edge
            || matches!(nodes[src], Node::Handoff)
            || matches!(nodes[dst], Node::Handoff)
        {
            new_preds[dst].insert(*dst_idx, (src, *src_idx));
            new_succs[src].insert(*src_idx, (dst, *dst_idx));
        } else {
            // Needs handoff inserted.
            // A -> H -> Z
            let hoff_id = nodes.insert(Node::Handoff);
            new_preds.insert(hoff_id, Default::default());
            new_succs.insert(hoff_id, Default::default());

            let zero_index = IndexInt {
                value: 0,
                span: Span::call_site(),
            };
            // A -> H.
            new_succs[src].insert(*src_idx, (hoff_id, zero_index));
            // A <- H.
            new_preds[hoff_id].insert(zero_index, (src, *src_idx));
            // H <- Z.
            new_preds[dst].insert(*dst_idx, (hoff_id, zero_index));
            // H -> Z.
            new_succs[hoff_id].insert(zero_index, (dst, *dst_idx));
        }
    }

    // Determine node's subgraph and subgraph's nodes.
    // This list of nodes in each subgraph are to be in topological sort order.
    let (node_subgraph, subgraph_nodes) = {
        struct SubgraphTopoSort<'a> {
            nodes: &'a SlotMap<GraphNodeId, Node>,
            preds: &'a SecondaryMap<GraphNodeId, OutboundEdges>,
            node_union: &'a mut UnionFind<GraphNodeId>,
            marked: HashSet<GraphNodeId>,
            subgraph_nodes: SecondaryMap<GraphNodeId, Vec<GraphNodeId>>,
        }
        impl<'a> SubgraphTopoSort<'a> {
            pub fn visit(&mut self, node_id: GraphNodeId) {
                // Already marked.
                if self.marked.contains(&node_id) {
                    return;
                }
                // Ignore handoff nodes.
                if matches!(self.nodes[node_id], Node::Handoff) {
                    return;
                }

                self.marked.insert(node_id);

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
            }
        }

        let (marked, subgraph_nodes) = Default::default();
        let mut sg_topo_sort = SubgraphTopoSort {
            nodes: &nodes,
            preds: &new_preds,
            node_union: &mut node_union,
            marked,
            subgraph_nodes,
        };
        for node_id in nodes.keys() {
            sg_topo_sort.visit(node_id);
        }

        // For a `NodeId`, what `SubgraphId` does it belong to.
        let mut node_subgraph: SecondaryMap<GraphNodeId, GraphSubgraphId> = Default::default();
        // For a `SubgraphId`, what `NodeId`s belong to it.
        let mut subgraph_nodes: SlotMap<GraphSubgraphId, Vec<GraphNodeId>> = Default::default();
        // Populate above.
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

    (new_preds, new_succs, node_subgraph, subgraph_nodes)
}

fn find_subgraph_strata(
    nodes: &mut SlotMap<GraphNodeId, Node>,
    new_preds: &mut AdjList,
    new_succs: &mut AdjList,
    node_subgraph: &mut SecondaryMap<GraphNodeId, GraphSubgraphId>,
    subgraph_nodes: &mut SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
    barrier_crossers: &Vec<(GraphNodeId, GraphNodeId, IndexInt, InputBarrier)>,
) -> SecondaryMap<GraphSubgraphId, usize> {
    // Determine subgraphs's stratum number.
    // Find SCCs ignoring negative edges, then do TopoSort on the resulting DAG.

    // Generate subgraph graph.
    let mut subgraph_preds: HashMap<GraphSubgraphId, Vec<GraphSubgraphId>> = Default::default();
    let mut subgraph_succs: HashMap<GraphSubgraphId, Vec<GraphSubgraphId>> = Default::default();

    for (node_id, node) in nodes.iter() {
        if matches!(node, Node::Handoff) {
            for &(pred, _) in new_preds[node_id].values() {
                let pred_sg = node_subgraph[pred];
                for &(succ, succ_idx) in new_succs[node_id].values() {
                    if barrier_crossers.contains(&(pred, succ, succ_idx, InputBarrier::Epoch)) {
                        continue;
                    }
                    let succ_sg = node_subgraph[succ];
                    subgraph_preds.entry(succ_sg).or_default().push(pred_sg);
                    subgraph_succs.entry(pred_sg).or_default().push(succ_sg);
                }
            }
        }
    }

    let scc = {
        // https://en.wikipedia.org/wiki/Kosaraju%27s_algorithm
        fn visit(
            succs: &HashMap<GraphSubgraphId, Vec<GraphSubgraphId>>,
            u: GraphSubgraphId,
            seen: &mut HashSet<GraphSubgraphId>,
            stack: &mut Vec<GraphSubgraphId>,
        ) {
            if seen.insert(u) {
                for &v in succs.get(&u).into_iter().flatten() {
                    visit(succs, v, seen, stack);
                }
                stack.push(u);
            }
        }

        let (mut seen, mut stack) = Default::default();
        for sg in subgraph_nodes.keys() {
            visit(&subgraph_succs, sg, &mut seen, &mut stack);
        }
        std::mem::drop(seen);

        fn assign(
            preds: &HashMap<GraphSubgraphId, Vec<GraphSubgraphId>>,
            u: GraphSubgraphId,
            root: GraphSubgraphId,
            components: &mut HashMap<GraphSubgraphId, GraphSubgraphId>,
        ) {
            if let Entry::Vacant(vacant_entry) = components.entry(u) {
                vacant_entry.insert(root);
                for &v in preds.get(&u).into_iter().flatten() {
                    assign(preds, v, root, components);
                }
            }
        }

        let mut components = Default::default();
        for sg in stack.into_iter().rev() {
            assign(&subgraph_preds, sg, sg, &mut components);
        }

        components
    };

    let topo_sort_order = {
        // Condensed each SCC into a single node for toposort.
        let condensed_preds: HashMap<GraphSubgraphId, Vec<GraphSubgraphId>> = subgraph_preds
            .iter()
            .map(|(u, preds)| (scc[u], preds.iter().map(|v| scc[v]).collect()))
            .collect();

        let (marked, order) = Default::default();
        let mut topo_sort = TopoSort {
            preds_fn: |v| condensed_preds.get(&v).into_iter().flatten().cloned(),
            marked,
            order,
        };
        for sg in subgraph_nodes.keys() {
            topo_sort.visit(sg);
        }

        topo_sort.order
    };

    let mut subgraph_stratum: SecondaryMap<GraphSubgraphId, usize> = topo_sort_order
        .into_iter()
        .enumerate()
        .map(|(n, sg_id)| (sg_id, n))
        .collect();

    let max_stratum = subgraph_stratum.values().cloned().max().unwrap_or(0) + 1;
    for &(src, dst, dst_idx, input_barrier) in barrier_crossers.iter() {
        let src_sg = node_subgraph[src];
        let dst_sg = node_subgraph[dst];
        let src_stratum = subgraph_stratum[src_sg];
        let dst_stratum = subgraph_stratum[dst_sg];
        match input_barrier {
            InputBarrier::None => unreachable!("No barrier"),
            InputBarrier::Stratum => {
                // Any negative edges which go onto the same or previous stratum are bad.
                // Indicates an unbroken negative cycle.
                if dst_stratum <= src_stratum {
                    dst_idx
                        .span()
                        .unwrap()
                        .error("Connection creates an unbroken negative cycle.")
                        .emit();
                }
            }
            InputBarrier::Epoch => {
                // If epoch edge goes foreward, need to buffer. (TODO: use a different kind of handoff).
                if src_stratum <= dst_stratum {
                    // Need to inject.
                    let new_node_id = nodes.insert(Node::Operator(parse_quote! { identity() }));
                    let new_subgraph_id = subgraph_nodes.insert(vec![new_node_id]);
                    subgraph_stratum.insert(new_subgraph_id, max_stratum);
                    node_subgraph.insert(new_node_id, new_subgraph_id);

                    let new_hoff_id = nodes.insert(Node::Handoff);

                    {
                        new_preds.insert(new_node_id, Default::default());
                        new_succs.insert(new_node_id, Default::default());
                        new_preds.insert(new_hoff_id, Default::default());
                        new_succs.insert(new_hoff_id, Default::default());
                    }

                    // Before: A (src) -> H -> B (dst)
                    // After: A -> H' -> X -> H -> B
                    {
                        let ii0 = IndexInt {
                            value: 0,
                            span: Span::call_site(),
                        };

                        let (old_hoff_id, _) = new_preds[dst][&dst_idx];
                        // X <- H
                        let (a_id, a_idx) = new_preds[old_hoff_id]
                            .insert(ii0, (new_node_id, ii0))
                            .expect("Handoff should be connected");

                        // A <-> H'
                        new_succs[a_id]
                            .insert(a_idx, (new_hoff_id, ii0))
                            .expect("A should be connected to old handoff");
                        new_preds[new_hoff_id].insert(ii0, (a_id, ii0));

                        //  H' <-> X -> H
                        new_succs[new_hoff_id].insert(ii0, (new_node_id, ii0));
                        new_preds[new_node_id].insert(ii0, (new_hoff_id, ii0));
                        new_succs[new_node_id].insert(ii0, (old_hoff_id, ii0));
                    }
                }
            }
        }
    }

    subgraph_stratum
}

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
    for edge in iter_edges(&new_succs) {
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

impl From<FlatGraph> for PartitionedGraph {
    fn from(mut flat_graph: FlatGraph) -> Self {
        // Pairs of node IDs which cross stratums or epochs and therefore cannot be in the same subgraph.
        let barrier_crossers = iter_edges(&flat_graph.succs)
            .filter_map(|((src, _src_idx), (dst, dst_idx))| {
                if let Node::Operator(dst_operator) = &flat_graph.nodes[dst] {
                    let dst_name = &*dst_operator.name_string();
                    OPERATORS
                        .iter()
                        .find(|&op| dst_name == op.name)
                        .map(|op_constraints| {
                            let input_barrier = (op_constraints.input_barrier_fn)(dst_idx.value);
                            (src, dst, *dst_idx, input_barrier)
                        })
                } else {
                    None
                }
            })
            .filter(|&(_, _, _, input_barrier)| InputBarrier::None != input_barrier)
            .collect::<Vec<_>>();

        let (mut new_preds, mut new_succs, mut node_subgraph, mut subgraph_nodes) = find_subgraphs(
            &mut flat_graph.nodes,
            &mut flat_graph.preds,
            &mut flat_graph.succs,
            &barrier_crossers,
        );

        let subgraph_stratum = find_subgraph_strata(
            &mut flat_graph.nodes,
            &mut new_preds,
            &mut new_succs,
            &mut node_subgraph,
            &mut subgraph_nodes,
            &barrier_crossers,
        );

        let (subgraph_recv_handoffs, subgraph_send_handoffs) = find_subgraph_handoffs(
            &flat_graph.nodes,
            &new_succs,
            &node_subgraph,
            &subgraph_nodes,
        );

        PartitionedGraph {
            nodes: flat_graph.nodes,
            preds: new_preds,
            succs: new_succs,
            node_subgraph,

            subgraph_nodes,
            subgraph_stratum,
            subgraph_recv_handoffs,
            subgraph_send_handoffs,
        }
    }
}

pub(crate) fn iter_edges(
    succs: &SecondaryMap<GraphNodeId, OutboundEdges>,
) -> impl '_ + Iterator<Item = (EdgePortRef, EdgePortRef)> {
    succs.iter().flat_map(|(src, succs)| {
        succs
            .iter()
            .map(move |(src_idx, (dst, dst_idx))| ((src, src_idx), (*dst, dst_idx)))
    })
}
