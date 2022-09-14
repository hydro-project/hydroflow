//! Graph representation stages for Hydroflow graphs.

use std::collections::BTreeMap;
use std::hash::Hash;

use proc_macro2::Span;
use slotmap::{new_key_type, SecondaryMap};
use syn::spanned::Spanned;

use crate::parse::{IndexInt, Operator};
use crate::pretty_span::PrettySpan;

pub mod di_mul_graph;
pub mod flat_graph;
pub mod flat_to_partitioned;
pub mod graph_algorithms;
pub mod ops;
pub mod partitioned_graph;
pub mod serde_graph;

new_key_type! {
    /// ID to identify a node (operator or handoff) in both [`flat_graph::FlatGraph`]
    /// and [`partitioned_graph::PartitionedGraph`].
    pub struct GraphNodeId;

    /// ID to identify an edge.
    pub struct GraphEdgeId;

    /// ID to identify a subgraph in [`partitioned_graph::PartitionedGraph`].
    pub struct GraphSubgraphId;
}

pub type EdgePort = (GraphNodeId, IndexInt);
pub type EdgePortRef<'a> = (GraphNodeId, &'a IndexInt);
/// BTreeMap is used to ensure iteration order matches `IndexInt` order.
pub type OutboundEdges = BTreeMap<IndexInt, EdgePort>;

type AdjList = SecondaryMap<GraphNodeId, OutboundEdges>;

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
                // (1, 1) =>
                _both_unary => None,
            },
        },
        Node::Handoff => Some(Color::Hoff),
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
