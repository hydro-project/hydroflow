use std::collections::HashMap;

use quote::ToTokens;
use slotmap::{Key, SecondaryMap, SlotMap};
use syn::LitInt;

use super::{flat_graph::FlatGraph, EdgePort, EdgePortRef, Node, NodeId, SubgraphId};

#[derive(Default)]
#[allow(dead_code)] // TODO(mingwei): remove when no longer needed.
pub struct PartitionedGraph {
    pub(crate) nodes: SlotMap<NodeId, Node>,
    pub(crate) preds: SecondaryMap<NodeId, HashMap<LitInt, EdgePort>>,
    pub(crate) succs: SecondaryMap<NodeId, HashMap<LitInt, EdgePort>>,

    pub(crate) node_subgraph: SecondaryMap<NodeId, SubgraphId>,
    pub(crate) subgraph_nodes: SlotMap<SubgraphId, Vec<NodeId>>,
}
impl PartitionedGraph {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_flat_graph(flat_graph: FlatGraph) -> Self {
        flat_graph.into()
    }

    pub fn edges(&self) -> impl '_ + Iterator<Item = (EdgePortRef, EdgePortRef)> {
        super::iter_edges(&self.succs)
    }

    pub fn subgraphs(&self) -> slotmap::basic::Iter<'_, SubgraphId, Vec<NodeId>> {
        self.subgraph_nodes.iter()
    }

    pub fn mermaid_string(&self) -> String {
        let mut string = String::new();
        self.write_mermaid(&mut string).unwrap();
        string
    }

    pub fn write_mermaid(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(write, "flowchart TB")?;
        for (subgraph_id, node_ids) in self.subgraphs() {
            writeln!(write, "    subgraph sg_{}", subgraph_id.data().as_ffi())?;
            for &node_id in node_ids.iter() {
                match &self.nodes[node_id] {
                    Node::Operator(operator) => {
                        writeln!(
                            write,
                            r#"        {}["{}"]"#,
                            node_id.data().as_ffi(),
                            operator
                                .to_token_stream()
                                .to_string()
                                .replace('&', "&amp;")
                                .replace('<', "&lt;")
                                .replace('>', "&gt;")
                                .replace('"', "&quot;"),
                        )?;
                    }
                    Node::Handoff => {
                        // writeln!(write, r#"        {}{{"handoff"}}"#, node_id.data().as_ffi())
                    }
                }
            }
            writeln!(write, "    end")?;
        }
        writeln!(write)?;
        for (node_id, node) in self.nodes.iter() {
            if matches!(node, Node::Handoff) {
                writeln!(write, r#"    {}{{"handoff"}}"#, node_id.data().as_ffi())?;
            }
        }
        writeln!(write)?;
        for ((src, _src_idx), (dst, _dst_idx)) in self.edges() {
            writeln!(
                write,
                "    {}-->{}",
                src.data().as_ffi(),
                dst.data().as_ffi()
            )?;
        }
        Ok(())
    }
}
