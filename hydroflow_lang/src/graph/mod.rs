//! Graph representation stages for Hydroflow graphs.

use std::hash::Hash;

use proc_macro2::{Span, TokenStream};
use slotmap::new_key_type;
use syn::spanned::Spanned;

use crate::parse::{ArrowConnector, Operator};
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

/// Helper struct for port indices which keeps span information for elided ports.
#[derive(Clone, Debug)]
pub enum PortIndexValue {
    Tokens(TokenStream),
    Elided(Span),
}
impl PortIndexValue {
    pub fn from_arrow_connector(arrow_connector: ArrowConnector) -> (Self, Self) {
        let src = arrow_connector
            .src
            .map(|idx| idx.index.into())
            .unwrap_or_else(|| Self::Elided(arrow_connector.arrow.span()));
        let dst = arrow_connector
            .dst
            .map(|idx| idx.index.into())
            .unwrap_or_else(|| Self::Elided(arrow_connector.arrow.span()));
        (src, dst)
    }
    pub fn is_specified(&self) -> bool {
        !matches!(self, Self::Elided(_))
    }
}
impl From<TokenStream> for PortIndexValue {
    fn from(tokens: TokenStream) -> Self {
        Self::Tokens(tokens)
    }
}
impl Spanned for PortIndexValue {
    fn span(&self) -> Span {
        match self {
            PortIndexValue::Tokens(x) => x.span(),
            PortIndexValue::Elided(span) => *span,
        }
    }
}
impl PartialEq for PortIndexValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Tokens(l0), Self::Tokens(r0)) => l0.to_string() == r0.to_string(),
            (Self::Elided(_), Self::Elided(_)) => true,
            _else => false,
        }
    }
}
impl Eq for PortIndexValue {}
impl PartialOrd for PortIndexValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Tokens(s), Self::Tokens(o)) => s.to_string().partial_cmp(&o.to_string()),
            (Self::Elided(_), Self::Elided(_)) => Some(std::cmp::Ordering::Equal),
            (_, Self::Elided(_)) => Some(std::cmp::Ordering::Less),
            (Self::Elided(_), _) => Some(std::cmp::Ordering::Greater),
        }
    }
}
impl Ord for PortIndexValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Tokens(s), Self::Tokens(o)) => s.to_string().cmp(&o.to_string()),
            (Self::Elided(_), Self::Elided(_)) => std::cmp::Ordering::Equal,
            (_, Self::Elided(_)) => std::cmp::Ordering::Less,
            (Self::Elided(_), _) => std::cmp::Ordering::Greater,
        }
    }
}
