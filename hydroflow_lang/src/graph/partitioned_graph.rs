use std::fmt::Debug;
use std::iter::FusedIterator;

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use slotmap::{Key, SecondaryMap, SlotMap, SparseSecondaryMap};
use syn::spanned::Spanned;

use crate::diagnostic::Diagnostic;

use super::ops::{
    find_op_op_constraints, DelayType, OperatorWriteOutput, WriteContextArgs, OPERATORS,
};
use super::serde_graph::{SerdeEdge, SerdeGraph};
use super::{
    get_operator_generics, node_color, Color, DiMulGraph, FlatGraph, FlatGraphExploded,
    GraphEdgeId, GraphNodeId, GraphSubgraphId, Node, OperatorInstance, PortIndexValue, CONTEXT,
    HANDOFF_NODE_STR, HYDROFLOW,
};

#[derive(Default)]
#[allow(dead_code)] // TODO(mingwei): remove when no longer needed.
pub struct PartitionedGraph {
    /// Each node (operator or handoff).
    pub(crate) nodes: SlotMap<GraphNodeId, Node>,
    /// Instance data corresponding to each operator node.
    pub(super) operator_instances: SecondaryMap<GraphNodeId, OperatorInstance>,
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
    /// Internal handoffs
    pub(crate) subgraph_internal_handoffs: SecondaryMap<GraphSubgraphId, Vec<GraphNodeId>>,
    /// The modality of each non-handoff node (Push or Pull)
    pub(crate) node_color: SparseSecondaryMap<GraphNodeId, Color>,

    /// What variable name each graph node belongs to (if any).
    pub(crate) node_varnames: SparseSecondaryMap<GraphNodeId, Ident>,
}
impl PartitionedGraph {
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a `PartitionedGraph` without partitions from the `FlatGraph`.
    pub(crate) fn unpartitioned_from_flat_graph(flat_graph: FlatGraph) -> Self {
        let FlatGraphExploded {
            nodes,
            operator_instances,
            graph,
            ports,
            node_varnames,
        } = flat_graph.explode();
        Self {
            nodes,
            operator_instances,
            graph,
            ports,

            node_varnames,

            ..Default::default()
        }
    }

    #[allow(clippy::result_unit_err)]
    pub fn from_flat_graph(flat_graph: FlatGraph) -> Result<Self, Diagnostic> {
        flat_graph.try_into()
    }

    /// Get a node with its operator instance (if applicable).
    pub fn node(&self, node_id: GraphNodeId) -> (&Node, Option<&OperatorInstance>) {
        (&self.nodes[node_id], self.operator_instances.get(node_id))
    }

    /// Iterator over `(GraphNodeId, &Node)` pairs.
    pub fn nodes(&self) -> slotmap::basic::Iter<GraphNodeId, Node> {
        self.nodes.iter()
    }

    /// Get edge: `(src GraphNodeId, src &PortIndexValue, dst GraphNodeId, dst &PortIndexValue))`.
    pub fn edge(
        &self,
        edge_id: GraphEdgeId,
    ) -> (GraphNodeId, &PortIndexValue, GraphNodeId, &PortIndexValue) {
        let (src, dst) = self.graph.edge(edge_id).expect("Edge not found");
        let (src_port, dst_port) = &self.ports[edge_id];
        (src, src_port, dst, dst_port)
    }

    /// Iterator over all edges: `(GraphEdgeId, (src GraphNodeId, src &PortIndexValue, dst GraphNodeId, dst &PortIndexValue))`.
    pub fn edges(
        &self,
    ) -> impl '_
           + Iterator<
        Item = (
            GraphEdgeId,
            (GraphNodeId, &PortIndexValue, GraphNodeId, &PortIndexValue),
        ),
    >
           + ExactSizeIterator
           + FusedIterator
           + Clone
           + Debug {
        self.graph.edges().map(|(edge_id, (src, dst))| {
            let (src_port, dst_port) = &self.ports[edge_id];
            (edge_id, (src, src_port, dst, dst_port))
        })
    }

    /// Successors, iterator of `(&PortIndexValue, GraphNodeId)` of outgoing edges.
    /// `PortIndexValue` for the port coming out of `src`.
    pub fn successors(
        &self,
        src: GraphNodeId,
    ) -> impl '_
           + Iterator<Item = (&PortIndexValue, GraphNodeId)>
           + DoubleEndedIterator
           + FusedIterator
           + Clone
           + Debug {
        self.graph
            .successors(src)
            .map(|(e, v)| (&self.ports[e].0, v))
    }

    /// Predecessors, iterator of `(&PortIndexValue, GraphNodeId)` of incoming edges.
    /// `PortIndexValue` for the port going into `dst`.
    pub fn predecessors(
        &self,
        dst: GraphNodeId,
    ) -> impl '_
           + Iterator<Item = (&PortIndexValue, GraphNodeId)>
           + DoubleEndedIterator
           + FusedIterator
           + Clone
           + Debug {
        self.graph
            .predecessors(dst)
            .map(|(e, v)| (&self.ports[e].1, v))
    }

    /// Degree into a node.
    pub fn degree_in(&self, dst: GraphNodeId) -> usize {
        self.graph.degree_in(dst)
    }

    /// Degree out of a node.
    pub fn degree_out(&self, src: GraphNodeId) -> usize {
        self.graph.degree_out(src)
    }

    /// `edge`: (src, dst, dst_idx)
    ///
    /// Before: A (src) ------------> B (dst)
    /// After:  A (src) -> X (new) -> B (dst)
    ///
    /// Returns the ID of X & ID of edge OUT of X.
    pub fn insert_intermediate_node(
        &mut self,
        edge_id: GraphEdgeId,
        new_node: Node,
    ) -> (GraphNodeId, GraphEdgeId) {
        let span = Some(new_node.span());

        // Make corresponding operator instance (if `node` is an operator).
        let op_inst_opt = 'oc: {
            let Node::Operator(operator) = &new_node else { break 'oc None; };
            let Some(op_constraints) = find_op_op_constraints(operator) else { break 'oc None; };
            let (input_port, output_port) = self.ports.get(edge_id).cloned().unwrap();
            let generics = get_operator_generics(
                &mut Vec::new(), /* TODO(mingwei) diagnostics */
                operator,
            );
            Some(OperatorInstance {
                op_constraints,
                input_ports: vec![input_port],
                output_ports: vec![output_port],
                generics,
                arguments: operator.args.clone(),
            })
        };

        // Insert new `node`.
        let node_id = self.nodes.insert(new_node);
        // Insert corresponding `OperatorInstance` if applicable.
        if let Some(op_inst) = op_inst_opt {
            self.operator_instances.insert(node_id, op_inst);
        }
        // Update edges to insert node within `edge_id`.
        let (e0, e1) = self
            .graph
            .insert_intermediate_vertex(node_id, edge_id)
            .unwrap();

        // Update corresponding ports.
        let (src_idx, dst_idx) = self.ports.remove(edge_id).unwrap();
        self.ports
            .insert(e0, (src_idx, PortIndexValue::Elided(span)));
        self.ports
            .insert(e1, (PortIndexValue::Elided(span), dst_idx));

        (node_id, e1)
    }

    pub fn serde_string(&self) -> String {
        let mut string = String::new();
        self.write_serde_graph(&mut string).unwrap();
        string
    }

    pub fn node_id_as_string(&self, node_id: GraphNodeId, is_pred: bool) -> String {
        match &self.nodes[node_id] {
            Node::Operator(_) => format!("op_{:?}", node_id.data()),
            Node::Handoff { .. } => format!(
                "hoff_{:?}_{}",
                node_id.data(),
                if is_pred { "recv" } else { "send" }
            ),
        }
    }

    pub fn node_id_as_ident(&self, node_id: GraphNodeId, is_pred: bool) -> Ident {
        let name = self.node_id_as_string(node_id, is_pred);
        let span = match (is_pred, &self.nodes[node_id]) {
            (_, Node::Operator(operator)) => operator.span(),
            (true, &Node::Handoff { src_span, .. }) => src_span,
            (false, &Node::Handoff { dst_span, .. }) => dst_span,
        };
        Ident::new(&*name, span)
    }

    pub fn as_code(&self, root: TokenStream, include_type_guards: bool) -> TokenStream {
        let hf = &Ident::new(HYDROFLOW, Span::call_site());

        let handoffs = self
            .nodes
            .iter()
            .filter_map(|(node_id, node)| match node {
                Node::Operator(_) => None,
                &Node::Handoff { src_span, dst_span } => Some((node_id, (src_span, dst_span))),
            })
            .map(|(node_id, (src_span, dst_span))| {
                let ident_send = Ident::new(&*format!("hoff_{:?}_send", node_id.data()), dst_span);
                let ident_recv = Ident::new(&*format!("hoff_{:?}_recv", node_id.data()), src_span);
                let hoff_name = Literal::string(&*format!("handoff {:?}", node_id));
                quote! {
                    let (#ident_send, #ident_recv) =
                        #hf.make_edge::<_, #root::scheduled::handoff::VecHandoff<_>>(#hoff_name);
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
                                matches!(self.nodes[node_id], Node::Handoff { .. }),
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
                        assert!(matches!(node, Node::Operator(_)), "Handoffs are not part of subgraphs.");
                        let op_inst = &self.operator_instances[node_id];

                        let op_span = node.span();
                        let op_name = op_inst.op_constraints.name;
                        let op_constraints = OPERATORS
                            .iter()
                            .find(|op| op_name == op.name)
                            .unwrap_or_else(|| panic!("Failed to find op: {}", op_name));

                        let ident = self.node_id_as_ident(node_id, false);

                        {
                            // TODO clean this up.
                            // Collect input arguments (predecessors).
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

                            let is_pull = idx < pull_to_push_idx;

                            let context_args = WriteContextArgs {
                                root: &root,
                                context: &Ident::new(CONTEXT, op_span),
                                hydroflow: &Ident::new(HYDROFLOW, op_span),
                                subgraph_id,
                                node_id,
                                op_span,
                                ident: &ident,
                                is_pull,
                                inputs: &*inputs,
                                outputs: &*outputs,
                                op_name,
                                op_inst,
                            };

                            let write_result = (op_constraints.write_fn)(&context_args, &mut diagnostics);
                            let Ok(OperatorWriteOutput {
                                write_prologue,
                                write_iterator,
                                write_iterator_after,
                            }) = write_result else {
                                continue;
                            };

                            op_prologue_code.push(write_prologue);
                            subgraph_op_iter_code.push(write_iterator);
                            if include_type_guards {
                                let fn_ident = format_ident!("check_{}", ident, span = op_span);
                                let pull_push_trait = if is_pull {
                                    quote_spanned! {op_span=>
                                        ::std::iter::Iterator<Item = Item>
                                    }
                                } else {
                                    quote_spanned! {op_span=>
                                        #root::pusherator::Pusherator<Item = Item>
                                    }
                                };
                                let iter_type_guard = quote_spanned! {op_span=>
                                    let #ident = {
                                        #[inline(always)]
                                        pub fn #fn_ident<Input: #pull_push_trait, Item>(input: Input) -> impl #pull_push_trait { input }
                                        #fn_ident( #ident )
                                    };
                                };
                                subgraph_op_iter_code.push(iter_type_guard);
                            }
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
                            #[inline(always)]
                            fn check_pivot_run<Pull: ::std::iter::Iterator<Item = Item>, Push: #root::pusherator::Pusherator<Item = Item>, Item>(pull: Pull, push: Push) {
                                #root::pusherator::pivot::Pivot::new(pull, push).run();
                            }
                            check_pivot_run(#pull_ident, #push_ident);
                        });
                    }
                };

                let hoff_name = Literal::string(&*format!("Subgraph {:?}", subgraph_id));
                let stratum = Literal::usize_unsuffixed(
                    self.subgraph_stratum.get(subgraph_id).cloned().unwrap_or(0),
                );
                let context = Ident::new(CONTEXT, Span::call_site());
                quote! {
                    #( #op_prologue_code )*

                    #hf.add_subgraph_stratified(
                        #hoff_name,
                        #stratum,
                        var_expr!( #( #recv_ports ),* ),
                        var_expr!( #( #send_ports ),* ),
                        move |#context, var_args!( #( #recv_ports ),* ), var_args!( #( #send_ports ),* )| {
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

                let mut #hf = #root::scheduled::graph::Hydroflow::new_with_graph(#serde_string);

                #( #handoffs )*
                #( #subgraphs )*

                #hf
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
        match &self.nodes[node_id] {
            Node::Operator(operator) => operator.to_token_stream().to_string(),
            Node::Handoff { .. } => HANDOFF_NODE_STR.to_string(),
        }
    }
    pub fn to_serde_graph(&self) -> SerdeGraph {
        // TODO(mingwei): Double initialization of SerdeGraph fields.
        let mut g = SerdeGraph::new();

        // add nodes
        for node_id in self.nodes.keys() {
            g.nodes.insert(node_id, self.node_to_txt(node_id));
        }

        // add edges
        for (edge_id, (src, dst)) in self.graph.edges() {
            let mut blocking = false;
            let the_ports = &self.ports[edge_id];
            if let Node::Operator(dest_op) = &self.nodes[dst] {
                let op_name = &*dest_op.name_string();
                let op_constraints = OPERATORS
                    .iter()
                    .find(|op| op_name == op.name)
                    .unwrap_or_else(|| panic!("Failed to find op: {}", op_name));
                if let Some(delay) = (op_constraints.input_delaytype_fn)(&the_ports.1) {
                    if delay == DelayType::Stratum {
                        blocking = true;
                    }
                }
            }
            let src_label = match &the_ports.0 {
                PortIndexValue::Path(path) => Some(path.to_token_stream().to_string()),
                PortIndexValue::Int(index) => Some(index.value.to_string()),
                _ => None,
            };
            let dst_label = match &the_ports.1 {
                PortIndexValue::Path(path) => Some(path.to_token_stream().to_string()),
                PortIndexValue::Int(index) => Some(index.value.to_string()),
                _ => None,
            };
            let label = match (src_label, dst_label) {
                (Some(l1), Some(l2)) => Some(format!("{} ~ {}", l1, l2)),
                (Some(l1), None) => Some(l1),
                (None, Some(l2)) => Some(l2),
                (None, None) => None,
            };

            let serde_edge = SerdeEdge {
                src,
                dst,
                blocking,
                label,
            };
            if let Some(adj) = g.edges.get_mut(src) {
                adj.push(serde_edge);
            } else {
                g.edges.insert(src, vec![serde_edge]);
            }
        }

        // add barrier_handoffs, i.e. handoffs that are *not* in the subgraph_recv_handoffs and
        // subgraph_send_handoffs for the same subgraph
        for sg in self.subgraph_recv_handoffs.keys() {
            let recvs = self.subgraph_recv_handoffs.get(sg).unwrap();
            let sends = self.subgraph_send_handoffs.get(sg).unwrap();
            for recv in recvs {
                if !sends.contains(recv) {
                    g.barrier_handoffs.insert(*recv, true);
                }
            }
        }

        // add subgraphs
        g.subgraph_nodes = self.subgraph_nodes.clone();
        g.subgraph_stratum = self.subgraph_stratum.clone();
        g.subgraph_internal_handoffs = self.subgraph_internal_handoffs.clone();

        // add varnames (sort for determinism).
        let mut varnames_sorted = self.node_varnames.iter().collect::<Vec<_>>();
        varnames_sorted.sort();
        for (node_id, varname_ident) in varnames_sorted {
            let node_ids = g
                .varname_nodes
                .entry(varname_ident.to_string())
                .or_default();
            node_ids.push(node_id);
        }

        g
    }

    pub fn write_serde_graph(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        let sg = self.to_serde_graph();
        writeln!(write, "{}", serde_json::to_string(&sg).unwrap())?;
        Ok(())
    }
}
