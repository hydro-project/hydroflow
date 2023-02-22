//! Graph representation stages for Hydroflow graphs.

use std::hash::Hash;

use proc_macro2::Span;
use quote::ToTokens;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;
use syn::spanned::Spanned;
use syn::ExprPath;

use crate::parse::{IndexInt, Operator, PortIndex, Ported};
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

/// Context identifier as a string.
const CONTEXT: &str = "context";

pub enum Node {
    Operator(Operator),
    Handoff { src_span: Span, dst_span: Span },
}
impl Spanned for Node {
    fn span(&self) -> Span {
        match self {
            Node::Operator(op) => op.span(),
            &Node::Handoff { src_span, dst_span } => src_span.join(dst_span).unwrap_or(src_span),
        }
    }
}
impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Operator(operator) => {
                write!(f, "Node::Operator({} span)", PrettySpan(operator.span()))
            }
            Self::Handoff { .. } => write!(f, "Node::Handoff"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
        Node::Handoff { .. } => Some(Color::Hoff),
    }
}

/// Helper struct for [`PortIndex`] which keeps span information for elided ports.
#[derive(Clone, Debug)]
pub enum PortIndexValue {
    Int(IndexInt),
    Path(ExprPath),
    Elided(Option<Span>),
}
impl PortIndexValue {
    pub fn from_ported<Inner>(ported: Ported<Inner>) -> (Self, Inner, Self)
    where
        Inner: Spanned,
    {
        let ported_span = Some(ported.inner.span());
        let port_inn = ported
            .inn
            .map(|idx| idx.index.into())
            .unwrap_or_else(|| Self::Elided(ported_span));
        let inner = ported.inner;
        let port_out = ported
            .out
            .map(|idx| idx.index.into())
            .unwrap_or_else(|| Self::Elided(ported_span));
        (port_inn, inner, port_out)
    }

    pub fn is_specified(&self) -> bool {
        !matches!(self, Self::Elided(_))
    }

    /// Return `Err(self)` if there is a conflict.
    pub fn combine(self, other: Self) -> Result<Self, Self> {
        if self.is_specified() {
            if other.is_specified() {
                Err(self)
            } else {
                Ok(self)
            }
        } else {
            Ok(other)
        }
    }
}
impl From<PortIndex> for PortIndexValue {
    fn from(value: PortIndex) -> Self {
        match value {
            PortIndex::Int(x) => Self::Int(x),
            PortIndex::Path(x) => Self::Path(x),
        }
    }
}
impl Spanned for PortIndexValue {
    fn span(&self) -> Span {
        match self {
            PortIndexValue::Int(x) => x.span(),
            PortIndexValue::Path(x) => x.span(),
            PortIndexValue::Elided(span) => span.unwrap_or_else(Span::call_site),
        }
    }
}
impl PartialEq for PortIndexValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(l0), Self::Int(r0)) => l0 == r0,
            (Self::Path(l0), Self::Path(r0)) => l0 == r0,
            (Self::Elided(_), Self::Elided(_)) => true,
            _else => false,
        }
    }
}
impl Eq for PortIndexValue {}
impl PartialOrd for PortIndexValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Int(s), Self::Int(o)) => s.partial_cmp(o),
            (Self::Path(s), Self::Path(o)) => s
                .to_token_stream()
                .to_string()
                .partial_cmp(&o.to_token_stream().to_string()),
            (Self::Elided(_), Self::Elided(_)) => Some(std::cmp::Ordering::Equal),
            (Self::Int(_), Self::Path(_)) => Some(std::cmp::Ordering::Less),
            (Self::Path(_), Self::Int(_)) => Some(std::cmp::Ordering::Greater),
            (_, Self::Elided(_)) => Some(std::cmp::Ordering::Less),
            (Self::Elided(_), _) => Some(std::cmp::Ordering::Greater),
        }
    }
}
impl Ord for PortIndexValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Int(s), Self::Int(o)) => s.cmp(o),
            (Self::Path(s), Self::Path(o)) => s
                .to_token_stream()
                .to_string()
                .cmp(&o.to_token_stream().to_string()),
            (Self::Elided(_), Self::Elided(_)) => std::cmp::Ordering::Equal,
            (Self::Int(_), Self::Path(_)) => std::cmp::Ordering::Less,
            (Self::Path(_), Self::Int(_)) => std::cmp::Ordering::Greater,
            (_, Self::Elided(_)) => std::cmp::Ordering::Less,
            (Self::Elided(_), _) => std::cmp::Ordering::Greater,
        }
    }
}
