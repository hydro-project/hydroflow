use std::collections::HashMap;

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, ToTokens};
use slotmap::{Key, SecondaryMap, SlotMap};
use syn::LitInt;

use super::flat_graph::FlatGraph;
use super::{EdgePort, EdgePortRef, Node, NodeId, SubgraphId};

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

    pub fn as_code(&self, root: TokenStream) -> TokenStream {
        let handoffs = self
            .nodes
            .iter()
            .filter(|(_node_id, node)| matches!(node, Node::Handoff))
            .map(|(node_id, _node)| {
                let ident_send = Ident::new(
                    &*format!("hoff{:?}_send", node_id.data()),
                    Span::call_site(),
                );
                let ident_recv = Ident::new(
                    &*format!("hoff{:?}_recv", node_id.data()),
                    Span::call_site(),
                );
                let hoff_name = Literal::string(&*format!("handoff {:?}", node_id));
                quote! {
                    let (#ident_send, #ident_recv) =
                        df.make_edge::<_, #root::scheduled::handoff::VecHandoff<_>>(#hoff_name);
                }
            });

        let subgraphs = self.subgraphs().map(|(subgraph_id, subgraph_nodes)| {
            let node_code = subgraph_nodes.iter().map(|&node_id| {
                let node = &self.nodes[node_id];
                let op = match node {
                    Node::Operator(op) => op,
                    Node::Handoff => unreachable!("Handoffs are not part of subgraphs."),
                };
                let ident = Ident::new(&*format!("op_{:?}", node_id.data()), Span::call_site());
                let preds = self.preds[node_id]
                    .values()
                    .map(|(pred_id, _)| pred_id)
                    .fold(String::new(), |mut str, pred_id| {
                        use std::fmt::Write;
                        write!(&mut str, "{:?}", pred_id).unwrap();
                        str
                    });
                let lit = Literal::string(&*format!("{} [{}]", op.to_token_stream(), preds));
                quote! {
                    let #ident = #lit;
                }
            });

            let hoff_name = Literal::string(&*format!("Subgraph {:?}", subgraph_id));
            quote! {
                df.add_subgraph(
                    #hoff_name,
                    tl!(),
                    tl!(),
                    move |context, tl!(), tl!()| {
                        #( #node_code )*
                    },
                );
            }
        });

        quote! {
            {
                use #root::tl;

                let mut df = #root::scheduled::graph::Hydroflow::new();
                #( #handoffs )*
                #( #subgraphs )*
                df
            }
        }
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
                            r#"        {:?}["{}"]"#,
                            node_id.data(),
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
                        // writeln!(write, r#"        {:?}{{"handoff"}}"#, node_id.data())
                    }
                }
            }
            writeln!(write, "    end")?;
        }
        writeln!(write)?;
        for (node_id, node) in self.nodes.iter() {
            if matches!(node, Node::Handoff) {
                writeln!(write, r#"    {:?}{{"handoff"}}"#, node_id.data())?;
            }
        }
        writeln!(write)?;
        for ((src, _src_idx), (dst, _dst_idx)) in self.edges() {
            writeln!(write, "    {:?}-->{:?}", src.data(), dst.data())?;
        }
        Ok(())
    }
}
