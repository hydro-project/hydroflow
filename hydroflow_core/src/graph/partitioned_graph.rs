use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, ToTokens};
use slotmap::{Key, SecondaryMap, SlotMap};
use syn::spanned::Spanned;

use super::flat_graph::FlatGraph;
use super::ops::OPERATORS;
use super::{node_color, Color, EdgePortRef, Node, NodeId, OutboundEdges, SubgraphId};

#[derive(Default)]
#[allow(dead_code)] // TODO(mingwei): remove when no longer needed.
pub struct PartitionedGraph {
    pub(crate) nodes: SlotMap<NodeId, Node>,
    pub(crate) preds: SecondaryMap<NodeId, OutboundEdges>,
    pub(crate) succs: SecondaryMap<NodeId, OutboundEdges>,
    pub(crate) node_subgraph: SecondaryMap<NodeId, SubgraphId>,

    pub(crate) subgraph_nodes: SlotMap<SubgraphId, Vec<NodeId>>,
    pub(crate) subgraph_recv_handoffs: SecondaryMap<SubgraphId, Vec<NodeId>>,
    pub(crate) subgraph_send_handoffs: SecondaryMap<SubgraphId, Vec<NodeId>>,
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

    pub fn mermaid_string(&self) -> String {
        let mut string = String::new();
        self.write_mermaid(&mut string).unwrap();
        string
    }

    pub fn node_id_as_ident(&self, node_id: NodeId, is_pred: bool) -> Ident {
        let name = match self.nodes[node_id] {
            Node::Operator(_) => {
                format!("op_{:?}", node_id.data())
            }
            Node::Handoff => {
                format!(
                    "hoff_{:?}_{}",
                    node_id.data(),
                    if is_pred { "recv" } else { "send" }
                )
            }
        };
        Ident::new(&*name, self.nodes[node_id].span())
    }

    pub fn as_code(&self, root: TokenStream) -> TokenStream {
        let handoffs = self
            .nodes
            .iter()
            .filter(|(_node_id, node)| matches!(node, Node::Handoff))
            .map(|(node_id, _node)| {
                let ident_send = Ident::new(
                    &*format!("hoff_{:?}_send", node_id.data()),
                    Span::call_site(),
                );
                let ident_recv = Ident::new(
                    &*format!("hoff_{:?}_recv", node_id.data()),
                    Span::call_site(),
                );
                let hoff_name = Literal::string(&*format!("handoff {:?}", node_id));
                quote! {
                    let (#ident_send, #ident_recv) =
                        df.make_edge::<_, #root::scheduled::handoff::VecHandoff<_>>(#hoff_name);
                }
            });

        let subgraphs = self
            .subgraph_nodes
            .iter()
            .map(|(subgraph_id, subgraph_nodes)| {
                let recv_ports: Vec<Ident> = self.subgraph_recv_handoffs[subgraph_id]
                    .iter()
                    .map(|&hoff_id| self.node_id_as_ident(hoff_id, true))
                    .collect();
                let send_ports: Vec<Ident> = self.subgraph_recv_handoffs[subgraph_id]
                    .iter()
                    .map(|&hoff_id| self.node_id_as_ident(hoff_id, false))
                    .collect();

                let recv_port_code = recv_ports
                    .iter()
                    .map(|ident| quote! { let #ident = #ident.take_inner().into_iter(); });
                let send_port_code = send_ports.iter().map(|ident| {
                    quote! {
                        let #ident = #root::compiled::for_each::ForEach::new(|v| {
                            #ident.give(Some(v));
                        });
                    }
                });

                let node_code = {
                    let nodes_iter = {
                        let pull_to_push_idx = subgraph_nodes
                            .iter()
                            .position(|&node_id| {
                                node_color(
                                    &self.nodes[node_id],
                                    self.preds[node_id].len(),
                                    self.succs[node_id].len(),
                                )
                                .map(|color| Color::Pull != color)
                                .unwrap_or(false)
                            })
                            .unwrap_or(subgraph_nodes.len());

                        let (pull_half, push_half) = subgraph_nodes.split_at(pull_to_push_idx);
                        pull_half.iter().chain(push_half.iter().rev())
                    };

                    nodes_iter.map(|&node_id| {
                        let node = &self.nodes[node_id];
                        let op = match node {
                            Node::Operator(op) => op,
                            Node::Handoff => unreachable!("Handoffs are not part of subgraphs."),
                        };

                        let op_name = &*op.name_string();
                        let op_constraints = OPERATORS
                            .iter()
                            .find(|op| op_name == op.name)
                            .unwrap_or_else(|| panic!("Failed to find op: {}", op_name));

                        let ident = self.node_id_as_ident(node_id, false);
                        // Note: `IndexInt` order is guaranteed by `BTreeMap` iteration order.
                        let inputs: Vec<_> = self.preds[node_id]
                            .values()
                            .map(|&(pred_id, _)| self.node_id_as_ident(pred_id, true))
                            .collect();
                        let outputs: Vec<_> = self.succs[node_id]
                            .values()
                            .map(|&(succ_id, _)| self.node_id_as_ident(succ_id, false))
                            .collect();

                        let code = (op_constraints.write_fn)(
                            &root,
                            &*inputs,
                            &*outputs,
                            op.type_arguments(),
                            &op.args,
                        );
                        quote! {
                            let #ident = #code;
                        }
                    })
                };

                let hoff_name = Literal::string(&*format!("Subgraph {:?}", subgraph_id));
                quote! {
                    df.add_subgraph(
                        #hoff_name,
                        tl!( #( #recv_ports ),* ),
                        tl!( #( #send_ports ),* ),
                        move |context, tl!( #( #recv_ports ),* ), tl!( #( #send_ports ),* )| {
                            #( #recv_port_code )*
                            #( #send_port_code )*
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
        for (subgraph_id, node_ids) in self.subgraph_nodes.iter() {
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
