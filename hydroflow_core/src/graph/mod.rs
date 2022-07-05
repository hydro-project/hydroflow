//! Graph representation stages for Hydroflow graphs.

use slotmap::new_key_type;
use syn::spanned::Spanned;
use syn::LitInt;

use crate::parse::Operator;
use crate::pretty_span::PrettySpan;

pub mod flat_graph;
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
