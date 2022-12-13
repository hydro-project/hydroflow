use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use slotmap::{Key, SecondaryMap, SlotMap};
use syn::spanned::Spanned;

use crate::diagnostic::Diagnostic;

use super::di_mul_graph::DiMulGraph;
use super::flat_graph::FlatGraph;
use super::ops::{OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, OPERATORS};
use super::serde_graph::SerdeGraph;
use super::{node_color, Color, GraphEdgeId, GraphNodeId, GraphSubgraphId, Node, PortIndexValue};

#[derive(Default)]
#[allow(dead_code)] // TODO(mingwei): remove when no longer needed.
pub struct PartitionedGraph {
    /// Each node (operator or handoff).
    pub(crate) nodes: SlotMap<GraphNodeId, Node>,
    /// Graph
    pub(crate) graph: DiMulGraph<GraphNodeId, GraphEdgeId>,
    /// Input and output port for each edge.
    pub(crate) ports: SecondaryMap<GraphEdgeId, (PortIndexValue, PortIndexValue)>,
    /// Which subgraph each node belongs to.
    pub(crate) node_subgraph: SecondaryMap<GraphNodeId, GraphSubgraphId>,

    /// Which nodes belong to each subgraph.
    pub(crate) subgraph_nodes: SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
    /// Which stratum each subgraph belongs to.
    pub(crate) subgraph_stratum: SecondaryMap<GraphSubgraphId, usize>,
    /// Which handoffs go into each subgraph.
    pub(crate) subgraph_recv_handoffs: SecondaryMap<GraphSubgraphId, Vec<GraphNodeId>>,
    /// Which handoffs go out of each subgraph.
    pub(crate) subgraph_send_handoffs: SecondaryMap<GraphSubgraphId, Vec<GraphNodeId>>,
}
impl PartitionedGraph {
    pub fn new() -> Self {
        Default::default()
    }

    #[allow(clippy::result_unit_err)]
    pub fn from_flat_graph(flat_graph: FlatGraph) -> Result<Self, Diagnostic> {
        flat_graph.try_into()
    }

    pub fn serde_string(&self) -> String {
        let mut string = String::new();
        self.write_serde_graph(&mut string).unwrap();
        string
    }

    pub fn node_id_as_string(&self, node_id: GraphNodeId, is_pred: bool) -> String {
        match &self.nodes[node_id] {
            Node::Operator(_) => format!("op_{:?}", node_id.data()),
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

        let mut diagnostics = Vec::new();

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
                        let #ident = #root::pusherator::for_each::ForEach::new(|v| {
                            #ident.give(Some(v));
                        });
                    }
                });

                let mut op_prologue_code = Vec::new();
                let mut subgraph_op_iter_code = Vec::new();
                let mut subgraph_op_iter_after_code = Vec::new();
                {
                    let pull_to_push_idx = subgraph_nodes
                        .iter()
                        .position(|&node_id| {
                            node_color(
                                &self.nodes[node_id],
                                self.graph.degree_in(node_id),
                                self.graph.degree_out(node_id),
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

                        let op_span = node.span();
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
                                op_span,
                            };

                            // TODO clean this up.
                            // Collect input arguments (predacessors).
                            let mut input_edges: Vec<(&PortIndexValue, GraphNodeId)> =
                                self.graph.predecessors(node_id)
                                    .map(|(edge_id, pred)| (&self.ports[edge_id].1, pred))
                                    .collect();
                            // Ensure sorted by port index.
                            input_edges.sort();

                            let inputs: Vec<Ident> = input_edges
                                .iter()
                                .map(|&(_port, pred)| self.node_id_as_ident(pred, true))
                                .collect();
                            let input_ports: Vec<&PortIndexValue> = input_edges.into_iter().map(|(port, _pred)| port).collect();

                            // Collect output arguments (successors).
                            let mut output_edges: Vec<(&PortIndexValue, GraphNodeId)> =
                                self.graph.successors(node_id)
                                    .map(|(edge_id, succ)| (&self.ports[edge_id].0, succ))
                                    .collect();
                            // Ensure sorted by port index.
                            output_edges.sort();

                            let outputs: Vec<Ident> = output_edges
                                .iter()
                                .map(|&(_port, succ)| self.node_id_as_ident(succ, false))
                                .collect();
                            let output_ports: Vec<&PortIndexValue> = output_edges.into_iter().map(|(port, _succ)| port).collect();

                            let is_pull = idx < pull_to_push_idx;

                            let iter_args = WriteIteratorArgs {
                                ident: &ident,
                                is_pull,
                                inputs: &*inputs,
                                outputs: &*outputs,
                                input_ports: &*input_ports,
                                output_ports: &*output_ports,
                                type_arguments: op.type_arguments(),
                                arguments: &op.args,
                                op_name: op_constraints.name,
                            };

                            let write_result = (op_constraints.write_fn)(&context_args, &iter_args, &mut diagnostics);
                            let Ok(OperatorWriteOutput {
                                write_prologue,
                                write_iterator,
                                write_iterator_after,
                            }) = write_result else {
                                continue;
                            };

                            let required_trait = if is_pull {
                                quote_spanned! {op_span=>
                                    std::iter::Iterator
                                }
                            } else {
                                quote_spanned! {op_span=>
                                    #root::pusherator::Pusherator
                                }
                            };
                            let iter_type_guard = quote_spanned! {op_span=>
                                #root::assert_var_impl!(#ident: #required_trait);
                            };

                            op_prologue_code.push(write_prologue);
                            subgraph_op_iter_code.push(write_iterator);
                            subgraph_op_iter_code.push(iter_type_guard);
                            subgraph_op_iter_after_code.push(write_iterator_after);
                        }
                    }

                    {
                        // Determine pull and push halves of the `Pivot`.
                        let pull_to_push_idx = pull_to_push_idx;
                        let pull_ident =
                            self.node_id_as_ident(subgraph_nodes[pull_to_push_idx - 1], false);

                        #[rustfmt::skip]
                        let push_ident = if let Some(&node_id) =
                            subgraph_nodes.get(pull_to_push_idx)
                        {
                            self.node_id_as_ident(node_id, false)
                        } else {
                            // Entire subgraph is pull (except for a single send/push handoff output).
                            assert_eq!(
                                1,
                                send_ports.len(),
                                "If entire subgraph is pull, should have only one handoff output. Do you have a loose `null()` or other degenerate pipeline somewhere?"
                            );
                            send_ports[0].clone()
                        };

                        // Pivot span is combination of pull and push spans (or if not possible, just take the push).
                        let pivot_span = pull_ident
                            .span()
                            .join(push_ident.span())
                            .unwrap_or_else(|| push_ident.span());
                        subgraph_op_iter_code.push(quote_spanned! {pivot_span=>
                            #root::pusherator::pivot::Pivot::new(#pull_ident, #push_ident).run();
                        });
                    }
                };

                let hoff_name = Literal::string(&*format!("Subgraph {:?}", subgraph_id));
                let stratum = Literal::usize_unsuffixed(
                    self.subgraph_stratum.get(subgraph_id).cloned().unwrap_or(0),
                );
                quote! {
                    #( #op_prologue_code )*

                    df.add_subgraph_stratified(
                        #hoff_name,
                        #stratum,
                        var_expr!( #( #recv_ports ),* ),
                        var_expr!( #( #send_ports ),* ),
                        move |context, var_args!( #( #recv_ports ),* ), var_args!( #( #send_ports ),* )| {
                            #( #recv_port_code )*
                            #( #send_port_code )*
                            #( #subgraph_op_iter_code )*
                            #( #subgraph_op_iter_after_code )*
                        },
                    );
                }
            });

        let serde_string = Literal::string(&*self.serde_string());
        let code = quote! {
            {
                use #root::{var_expr, var_args};

                let mut df = #root::scheduled::graph::Hydroflow::new_with_graph(#serde_string);

                #( #handoffs )*
                #( #subgraphs )*

                df
            }
        };

        diagnostics.iter().for_each(Diagnostic::emit);
        if diagnostics.iter().any(Diagnostic::is_error) {
            quote! { #root::scheduled::graph::Hydroflow::new() }
        } else {
            code
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
        // TODO(mingwei): Double initialization of SerdeGraph fields.
        let mut g = SerdeGraph::new();
        for (_edge_id, (src, dst)) in self.graph.edges() {
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
            g.subgraph_stratum = self.subgraph_stratum.clone();
        }
        g
    }

    pub fn write_serde_graph(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        let sg = self.to_serde_graph();
        writeln!(write, "{}", serde_json::to_string(&sg).unwrap())?;
        Ok(())
    }
}
