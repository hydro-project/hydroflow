use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, ToTokens};
use slotmap::{Key, SecondaryMap, SlotMap};
use syn::spanned::Spanned;

use super::flat_graph::FlatGraph;
use super::ops::{WriteContextArgs, WriteIteratorArgs, OPERATORS};
use super::serde_graph::SerdeGraph;
use super::{node_color, Color, EdgePortRef, GraphNodeId, GraphSubgraphId, Node, OutboundEdges};

#[derive(Default)]
#[allow(dead_code)] // TODO(mingwei): remove when no longer needed.
pub struct PartitionedGraph {
    pub(crate) nodes: SlotMap<GraphNodeId, Node>,
    pub(crate) preds: SecondaryMap<GraphNodeId, OutboundEdges>,
    pub(crate) succs: SecondaryMap<GraphNodeId, OutboundEdges>,
    pub(crate) node_subgraph: SecondaryMap<GraphNodeId, GraphSubgraphId>,

    pub(crate) subgraph_nodes: SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
    pub(crate) subgraph_recv_handoffs: SecondaryMap<GraphSubgraphId, Vec<GraphNodeId>>,
    pub(crate) subgraph_send_handoffs: SecondaryMap<GraphSubgraphId, Vec<GraphNodeId>>,
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

    pub fn serde_string(&self) -> String {
        let mut string = String::new();
        self.write_serde_graph(&mut string).unwrap();
        string
    }

    pub fn node_id_as_string(&self, node_id: GraphNodeId, is_pred: bool) -> String {
        match &self.nodes[node_id] {
            Node::Operator(_) => format!("op_{:?}", node_id.data()).into(),
            Node::Handoff => format!(
                "hoff_{:?}_{}",
                node_id.data(),
                if is_pred { "recv" } else { "send" }
            ),
        }
    }

    pub fn node_id_as_ident(&self, node_id: GraphNodeId, is_pred: bool) -> Ident {
        let name = self.node_id_as_string(node_id, is_pred);
        Ident::new(&*name, self.nodes[node_id].span())
    }

    pub fn tokenize(&self, _root: TokenStream) -> TokenStream {
        let t = self
            .nodes
            .values()
            .filter_map(|node| match node {
                Node::Operator(operator) => Some(operator),
                Node::Handoff => None,
            })
            .map(|operator| {
                let op_tokens = operator.to_token_stream();
                quote! { #op_tokens }
            });
        quote! {
            {
                #( quote::quote!{ #t } );*
            }
        }
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
                let send_ports: Vec<Ident> = self.subgraph_send_handoffs[subgraph_id]
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

                let mut op_prologue_code = Vec::new();
                let mut subgraph_op_iter_code = Vec::new();
                {
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
                    let nodes_iter = pull_half.iter().chain(push_half.iter().rev());

                    for (idx, &node_id) in nodes_iter.enumerate() {
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

                        {
                            let context_args = WriteContextArgs {
                                root: &root,
                                subgraph_id,
                                node_id,
                                ident: &ident,
                            };

                            // Note: `IndexInt` order is guaranteed by `BTreeMap` iteration order.
                            let inputs: Vec<_> = self.preds[node_id]
                                .values()
                                .map(|&(pred_id, _)| self.node_id_as_ident(pred_id, true))
                                .collect();
                            let outputs: Vec<_> = self.succs[node_id]
                                .values()
                                .map(|&(succ_id, _)| self.node_id_as_ident(succ_id, false))
                                .collect();

                            let iter_args = WriteIteratorArgs {
                                inputs: &*inputs,
                                outputs: &*outputs,
                                type_arguments: op.type_arguments(),
                                arguments: &op.args,
                                is_pull: idx < pull_to_push_idx,
                            };

                            op_prologue_code.push((op_constraints.write_prologue_fn)(
                                &context_args,
                                &iter_args,
                            ));

                            let iter_code =
                                (op_constraints.write_iterator_fn)(&context_args, &iter_args);

                            subgraph_op_iter_code.push(quote! {
                                let #ident = #iter_code;
                            });
                        }
                    }

                    {
                        let pull_to_push_idx = pull_to_push_idx;
                        let pull_ident =
                            self.node_id_as_ident(subgraph_nodes[pull_to_push_idx - 1], false);
                        let push_ident =
                            self.node_id_as_ident(subgraph_nodes[pull_to_push_idx], false);
                        subgraph_op_iter_code.push(quote! {
                            #root::compiled::pivot::Pivot::new(#pull_ident, #push_ident).run();
                        });
                    }
                };

                let hoff_name = Literal::string(&*format!("Subgraph {:?}", subgraph_id));
                quote! {
                    #( #op_prologue_code )*

                    df.add_subgraph(
                        #hoff_name,
                        tl!( #( #recv_ports ),* ),
                        tl!( #( #send_ports ),* ),
                        move |context, tl!( #( #recv_ports ),* ), tl!( #( #send_ports ),* )| {
                            #( #recv_port_code )*
                            #( #send_port_code )*
                            #( #subgraph_op_iter_code )*
                        },
                    );
                }
            });

        let serde_string = Literal::string(&*self.serde_string());
        quote! {
            {
                use #root::tl;

                let mut df = #root::scheduled::graph::Hydroflow::new_with_graph(#serde_string);

                #( #handoffs )*
                #( #subgraphs )*

                df
            }
        }
    }

    pub fn node_to_txt(&self, node_id: GraphNodeId) -> String {
        format!(
            "{}: {}",
            self.node_id_as_string(node_id, false),
            match &self.nodes[node_id] {
                Node::Operator(operator) => {
                    operator.to_token_stream().to_string()
                }
                Node::Handoff => {
                    "handoff".to_string()
                }
            }
        )
    }
    pub fn to_serde_graph(&self) -> SerdeGraph {
        let mut g = SerdeGraph::new();
        for ((src, _src_idx), (dst, _dst_idx)) in self.edges() {
            // add nodes
            g.nodes.insert(src, self.node_to_txt(src));
            g.nodes.insert(dst, self.node_to_txt(dst));

            // add handoffs
            if let Node::Handoff = &self.nodes[src] {
                g.handoffs.insert(src, true);
            }
            if let Node::Handoff = &self.nodes[dst] {
                g.handoffs.insert(dst, true);
            }

            // add edges
            match g.edges.get_mut(src) {
                Some(e) => {
                    e.push(dst);
                }
                None => {
                    g.edges.insert(src, vec![dst]);
                }
            }

            // add subgraphs
            g.subgraph_nodes = self.subgraph_nodes.clone();
        }
        g
    }

    pub fn write_serde_graph(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        let sg = self.to_serde_graph();
        writeln!(write, "{}", serde_json::to_string(&sg).unwrap())?;
        Ok(())
    }
}
