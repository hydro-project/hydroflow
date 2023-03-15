#![deny(missing_docs)]

use std::fmt::Debug;
use std::iter::FusedIterator;

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use serde::{Deserialize, Serialize};
use slotmap::{Key, SecondaryMap, SlotMap, SparseSecondaryMap};
use syn::spanned::Spanned;

use crate::diagnostic::{Diagnostic, Level};
use crate::pretty_span::{PrettyRowCol, PrettySpan};

use super::ops::{
    find_op_op_constraints, DelayType, OperatorWriteOutput, WriteContextArgs, OPERATORS,
};
use super::serde_graph::{SerdeEdge, SerdeGraph};
use super::{
    get_operator_generics, node_color, Color, DiMulGraph, GraphEdgeId, GraphNodeId,
    GraphSubgraphId, Node, OperatorInstance, PortIndexValue, Varname, CONTEXT, HANDOFF_NODE_STR,
    HYDROFLOW,
};

/// A graph representing a hydroflow dataflow graph (with or without subgraph partitioning,
/// stratification, and handoff insertion). This is a "meta" graph used for generating Rust source
/// code in macros from Hydroflow surface sytnax.
///
/// This struct has a lot of methods for manipulating the graph, vaguely grouped together in
/// separate `impl` blocks. You might notice a few particularly specific arbitray-seeming methods
/// in here--those are just what was needed for the compilation algorithms. If you need another
/// method then add it.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct HydroflowGraph {
    /// Each node (operator or handoff).
    nodes: SlotMap<GraphNodeId, Node>,
    /// Instance data corresponding to each operator node.
    /// This field will be empty after deserialization.
    #[serde(skip)]
    operator_instances: SecondaryMap<GraphNodeId, OperatorInstance>,
    /// Graph data structure (two-way adjacency list).
    graph: DiMulGraph<GraphNodeId, GraphEdgeId>,
    /// Input and output port for each edge.
    ports: SecondaryMap<GraphEdgeId, (PortIndexValue, PortIndexValue)>,
    /// Which subgraph each node belongs to.
    node_subgraph: SecondaryMap<GraphNodeId, GraphSubgraphId>,

    /// Which nodes belong to each subgraph.
    subgraph_nodes: SlotMap<GraphSubgraphId, Vec<GraphNodeId>>,
    /// Which stratum each subgraph belongs to.
    subgraph_stratum: SecondaryMap<GraphSubgraphId, usize>,

    /// What variable name each graph node belongs to (if any).
    node_varnames: SparseSecondaryMap<GraphNodeId, Varname>,
}
impl HydroflowGraph {
    /// Create a new empty `HydroflowGraph`.
    pub fn new() -> Self {
        Default::default()
    }
}
// Node methods.
impl HydroflowGraph {
    /// Get a node with its operator instance (if applicable).
    pub fn node(&self, node_id: GraphNodeId) -> &Node {
        self.nodes.get(node_id).expect("Node not found.")
    }

    /// Get the `OperatorInstance` for a given node. Node must be an operator and have an
    /// `OperatorInstance` present, otherwise will return `None`.
    ///
    /// Note that no operator instances will be persent after deserialization.
    pub fn node_op_inst(&self, node_id: GraphNodeId) -> Option<&OperatorInstance> {
        self.operator_instances.get(node_id)
    }

    /// Get subgraph for node.
    pub fn node_subgraph(&self, node_id: GraphNodeId) -> Option<GraphSubgraphId> {
        self.node_subgraph.get(node_id).copied()
    }

    /// Degree into a node, i.e. the number of predecessors.
    pub fn node_degree_in(&self, node_id: GraphNodeId) -> usize {
        self.graph.degree_in(node_id)
    }

    /// Degree out of a node, i.e. the number of successors.
    pub fn node_degree_out(&self, node_id: GraphNodeId) -> usize {
        self.graph.degree_out(node_id)
    }

    /// Successors, iterator of `(GraphEdgeId, GraphNodeId)` of outgoing edges.
    pub fn node_successors(
        &self,
        src: GraphNodeId,
    ) -> impl '_
           + Iterator<Item = (GraphEdgeId, GraphNodeId)>
           + DoubleEndedIterator
           + ExactSizeIterator
           + FusedIterator
           + Clone
           + Debug {
        self.graph.successors(src)
    }

    /// Predecessors, iterator of `(GraphEdgeId, GraphNodeId)` of incoming edges.
    pub fn node_predecessors(
        &self,
        dst: GraphNodeId,
    ) -> impl '_
           + Iterator<Item = (GraphEdgeId, GraphNodeId)>
           + DoubleEndedIterator
           + ExactSizeIterator
           + FusedIterator
           + Clone
           + Debug {
        self.graph.predecessors(dst)
    }

    /// Successor edges, iterator of `GraphEdgeId` of outgoing edges.
    pub fn node_successor_edges(
        &self,
        src: GraphNodeId,
    ) -> impl '_
           + Iterator<Item = GraphEdgeId>
           + DoubleEndedIterator
           + ExactSizeIterator
           + FusedIterator
           + Clone
           + Debug {
        self.graph.successor_edges(src)
    }

    /// Predecessor edges, iterator of `GraphEdgeId` of incoming edges.
    pub fn node_predecessor_edges(
        &self,
        src: GraphNodeId,
    ) -> impl '_
           + Iterator<Item = GraphEdgeId>
           + DoubleEndedIterator
           + ExactSizeIterator
           + FusedIterator
           + Clone
           + Debug {
        self.graph.predecessor_edges(src)
    }

    /// Successor nodes, iterator of `GraphNodeId`.
    pub fn node_successor_nodes(
        &self,
        src: GraphNodeId,
    ) -> impl '_
           + Iterator<Item = GraphNodeId>
           + DoubleEndedIterator
           + ExactSizeIterator
           + FusedIterator
           + Clone
           + Debug {
        self.graph.successor_vertices(src)
    }

    /// Predecessor edges, iterator of `GraphNodeId`.
    pub fn node_predecessor_nodes(
        &self,
        src: GraphNodeId,
    ) -> impl '_
           + Iterator<Item = GraphNodeId>
           + DoubleEndedIterator
           + ExactSizeIterator
           + FusedIterator
           + Clone
           + Debug {
        self.graph.predecessor_vertices(src)
    }

    /// Iterator of node IDs `GraphNodeId`.
    pub fn node_ids(&self) -> slotmap::basic::Keys<'_, GraphNodeId, Node> {
        self.nodes.keys()
    }

    /// Iterator over `(GraphNodeId, &Node)` pairs.
    pub fn nodes(&self) -> slotmap::basic::Iter<'_, GraphNodeId, Node> {
        self.nodes.iter()
    }

    /// Insert a node, assigning the given varname.
    pub fn insert_node(&mut self, node: Node, varname_opt: Option<Ident>) -> GraphNodeId {
        let node_id = self.nodes.insert(node);
        if let Some(varname) = varname_opt {
            self.node_varnames.insert(node_id, Varname(varname));
        }
        node_id
    }

    /// Insert an operator instance for the given node. Panics if already set.
    pub fn insert_node_op_inst(&mut self, node_id: GraphNodeId, op_inst: OperatorInstance) {
        assert!(matches!(self.nodes.get(node_id), Some(Node::Operator(_))));
        let old_inst = self.operator_instances.insert(node_id, op_inst);
        assert!(old_inst.is_none());
    }

    /// Assign all operator instances if not set. Write diagnostic messages/errors into `diagnostics`.
    pub fn insert_node_op_insts_all(&mut self, diagnostics: &mut Vec<Diagnostic>) {
        let mut op_insts = Vec::new();
        for (node_id, node) in self.nodes() {
            let Node::Operator(operator) = node else {
                continue;
            };
            if self.node_op_inst(node_id).is_some() {
                continue;
            };

            // Op constraints.
            let Some(op_constraints) = find_op_op_constraints(operator) else {
                diagnostics.push(Diagnostic::spanned(
                    operator.path.span(),
                    Level::Error,
                    format!("Unknown operator `{}`", operator.name_string()),
                ));
                continue;
            };

            // Input and output ports.
            let (input_ports, output_ports) = {
                let mut input_edges: Vec<(&PortIndexValue, GraphNodeId)> = self
                    .node_predecessors(node_id)
                    .map(|(edge_id, pred_id)| (self.edge_ports(edge_id).1, pred_id))
                    .collect();
                // Ensure sorted by port index.
                input_edges.sort();
                let input_ports: Vec<PortIndexValue> = input_edges
                    .into_iter()
                    .map(|(port, _pred)| port)
                    .cloned()
                    .collect();

                // Collect output arguments (successors).
                let mut output_edges: Vec<(&PortIndexValue, GraphNodeId)> = self
                    .node_successors(node_id)
                    .map(|(edge_id, succ)| (self.edge_ports(edge_id).0, succ))
                    .collect();
                // Ensure sorted by port index.
                output_edges.sort();
                let output_ports: Vec<PortIndexValue> = output_edges
                    .into_iter()
                    .map(|(port, _succ)| port)
                    .cloned()
                    .collect();

                (input_ports, output_ports)
            };

            // Generic arguments.
            let generics = get_operator_generics(diagnostics, operator);
            // Generic argument errors.
            {
                if !op_constraints
                    .persistence_args
                    .contains(&generics.persistence_args.len())
                {
                    diagnostics.push(Diagnostic::spanned(
                        generics.generic_args.span(),
                        Level::Error,
                        format!(
                            "`{}` should have {} persistence lifetime arguments, actually has {}.",
                            op_constraints.name,
                            op_constraints.persistence_args.human_string(),
                            generics.persistence_args.len()
                        ),
                    ));
                }
                if !op_constraints.type_args.contains(&generics.type_args.len()) {
                    diagnostics.push(Diagnostic::spanned(
                        generics.generic_args.span(),
                        Level::Error,
                        format!(
                            "`{}` should have {} generic type arguments, actually has {}.",
                            op_constraints.name,
                            op_constraints.type_args.human_string(),
                            generics.type_args.len()
                        ),
                    ));
                }
            }

            op_insts.push((
                node_id,
                OperatorInstance {
                    op_constraints,
                    input_ports,
                    output_ports,
                    generics,
                    arguments: operator.args.clone(),
                },
            ));
        }

        for (node_id, op_inst) in op_insts {
            self.insert_node_op_inst(node_id, op_inst);
        }
    }

    /// Inserts a node between two existing nodes connected by the given `edge_id`.
    ///
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
}
// Edge methods.
impl HydroflowGraph {
    /// Get the `src` and `dst` for an edge: `(src GraphNodeId, dst GraphNodeId)`.
    pub fn edge(&self, edge_id: GraphEdgeId) -> (GraphNodeId, GraphNodeId) {
        let (src, dst) = self.graph.edge(edge_id).expect("Edge not found.");
        (src, dst)
    }

    /// Get the source and destination ports for an edge: `(src &PortIndexValue, dst &PortIndexValue)`.
    pub fn edge_ports(&self, edge_id: GraphEdgeId) -> (&PortIndexValue, &PortIndexValue) {
        let (src_port, dst_port) = self.ports.get(edge_id).expect("Edge not found.");
        (src_port, dst_port)
    }

    /// Iterator of all edge IDs `GraphEdgeId`.
    pub fn edge_ids(&self) -> slotmap::basic::Keys<GraphEdgeId, (GraphNodeId, GraphNodeId)> {
        self.graph.edge_ids()
    }

    /// Iterator over all edges: `(GraphEdgeId, (src GraphNodeId, dst GraphNodeId))`.
    pub fn edges(
        &self,
    ) -> impl '_
           + Iterator<Item = (GraphEdgeId, (GraphNodeId, GraphNodeId))>
           + ExactSizeIterator
           + FusedIterator
           + Clone
           + Debug {
        self.graph.edges()
    }

    /// Insert an edge between nodes thru the given ports.
    pub fn insert_edge(
        &mut self,
        src: GraphNodeId,
        src_port: PortIndexValue,
        dst: GraphNodeId,
        dst_port: PortIndexValue,
    ) -> GraphEdgeId {
        let edge_id = self.graph.insert_edge(src, dst);
        self.ports.insert(edge_id, (src_port, dst_port));
        edge_id
    }
}
// Subgraph methods.
impl HydroflowGraph {
    /// Nodes belonging to the given subgraph.
    pub fn subgraph(&self, subgraph_id: GraphSubgraphId) -> &Vec<GraphNodeId> {
        self.subgraph_nodes
            .get(subgraph_id)
            .expect("Subgraph not found.")
    }

    /// Iterator over all subgraph IDs.
    pub fn subgraph_ids(&self) -> slotmap::basic::Keys<'_, GraphSubgraphId, Vec<GraphNodeId>> {
        self.subgraph_nodes.keys()
    }

    /// Iterator over all subgraphs, ID and members: `(GraphSubgraphId, Vec<GraphNodeId>)`.
    pub fn subgraphs(&self) -> slotmap::basic::Iter<'_, GraphSubgraphId, Vec<GraphNodeId>> {
        self.subgraph_nodes.iter()
    }

    /// Create a subgraph consisting of `node_ids`. Returns an error if any of the nodes are already in a subgraph.
    pub fn insert_subgraph(
        &mut self,
        node_ids: Vec<GraphNodeId>,
    ) -> Result<GraphSubgraphId, (GraphNodeId, GraphSubgraphId)> {
        // Check none are already in subgraphs
        for &node_id in node_ids.iter() {
            if let Some(&old_sg_id) = self.node_subgraph.get(node_id) {
                return Err((node_id, old_sg_id));
            }
        }
        Ok(self.subgraph_nodes.insert_with_key(|sg_id| {
            for &node_id in node_ids.iter() {
                self.node_subgraph.insert(node_id, sg_id);
            }
            node_ids
        }))
    }

    /// Removes a node from its subgraph. Returns true if the node was in a subgraph.
    pub fn remove_from_subgraph(&mut self, node_id: GraphNodeId) -> bool {
        if let Some(old_sg_id) = self.node_subgraph.remove(node_id) {
            self.subgraph_nodes[old_sg_id].retain(|&other_node_id| other_node_id != node_id);
            true
        } else {
            false
        }
    }

    /// Gets the stratum nubmer of the subgraph.
    pub fn subgraph_stratum(&self, sg_id: GraphSubgraphId) -> Option<usize> {
        self.subgraph_stratum.get(sg_id).copied()
    }

    /// Set subgraph's stratum number, returning the old value if exists.
    pub fn set_subgraph_stratum(
        &mut self,
        sg_id: GraphSubgraphId,
        stratum: usize,
    ) -> Option<usize> {
        self.subgraph_stratum.insert(sg_id, stratum)
    }

    /// Returns the the stratum number of the largest (latest) stratum (inclusive).
    pub fn max_stratum(&self) -> Option<usize> {
        self.subgraph_stratum.values().copied().max()
    }
}
// Display/output stuff.
impl HydroflowGraph {
    /// Helper to generate a deterministic `Ident` for the given node.
    fn node_as_ident(&self, node_id: GraphNodeId, is_pred: bool) -> Ident {
        let name = match &self.nodes[node_id] {
            Node::Operator(_) => format!("op_{:?}", node_id.data()),
            Node::Handoff { .. } => format!(
                "hoff_{:?}_{}",
                node_id.data(),
                if is_pred { "recv" } else { "send" }
            ),
        };
        let span = match (is_pred, &self.nodes[node_id]) {
            (_, Node::Operator(operator)) => operator.span(),
            (true, &Node::Handoff { src_span, .. }) => src_span,
            (false, &Node::Handoff { dst_span, .. }) => dst_span,
        };
        Ident::new(&*name, span)
    }

    /// Returns each subgraph's receive and send handoffs.
    /// `Map<GraphSubgraphId, (recv handoffs, send handoffs)>`
    fn helper_collect_subgraph_handoffs(
        &self,
    ) -> SecondaryMap<GraphSubgraphId, (Vec<GraphNodeId>, Vec<GraphNodeId>)> {
        // Get data on handoff src and dst subgraphs.
        let mut subgraph_handoffs: SecondaryMap<
            GraphSubgraphId,
            (Vec<GraphNodeId>, Vec<GraphNodeId>),
        > = self
            .subgraph_nodes
            .keys()
            .map(|k| (k, Default::default()))
            .collect();

        // For each handoff node, add it to the `send`/`recv` lists for the corresponding subgraphs.
        for (hoff_id, node) in self.nodes() {
            if !matches!(node, Node::Handoff { .. }) {
                continue;
            }
            // Receivers from the handoff. (Should really only be one).
            for (_edge, succ_id) in self.node_successors(hoff_id) {
                let succ_sg = self.node_subgraph(succ_id).unwrap();
                subgraph_handoffs[succ_sg].0.push(hoff_id);
            }
            // Senders into the handoff. (Should really only be one).
            for (_edge, pred_id) in self.node_predecessors(hoff_id) {
                let pred_sg = self.node_subgraph(pred_id).unwrap();
                subgraph_handoffs[pred_sg].1.push(hoff_id);
            }
        }

        subgraph_handoffs
    }

    /// Emit this `HydroflowGraph` as runnable Rust source code tokens.
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

        let subgraph_handoffs = self.helper_collect_subgraph_handoffs();

        let subgraphs = self
            .subgraph_nodes
            .iter()
            .map(|(subgraph_id, subgraph_nodes)| {
                let (recv_hoffs, send_hoffs) = &subgraph_handoffs[subgraph_id];
                let recv_ports: Vec<Ident> = recv_hoffs
                    .iter()
                    .map(|&hoff_id| self.node_as_ident(hoff_id, true))
                    .collect();
                let send_ports: Vec<Ident> = send_hoffs
                    .iter()
                    .map(|&hoff_id| self.node_as_ident(hoff_id, false))
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

                        let ident = self.node_as_ident(node_id, false);

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
                                .map(|&(_port, pred)| self.node_as_ident(pred, true))
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
                                .map(|&(_port, succ)| self.node_as_ident(succ, false))
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
                            self.node_as_ident(subgraph_nodes[pull_to_push_idx - 1], false);

                        #[rustfmt::skip]
                        let push_ident = if let Some(&node_id) =
                            subgraph_nodes.get(pull_to_push_idx)
                        {
                            self.node_as_ident(node_id, false)
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

        let serde_string = serde_json::to_string(&self).unwrap();
        // Newline left over from old code, snapshot outputs.
        let serde_string = Literal::string(&*serde_string);
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

    /// The `SerdeGraph` version of this, sent to the final runnable `Hydroflow` instance via
    /// serialization and deserialization.
    ///
    /// TODO(mingwei): ser/de `HydroflowGraph` directly.
    pub fn to_serde_graph(&self) -> SerdeGraph {
        // TODO(mingwei): Double initialization of SerdeGraph fields.
        let mut g = SerdeGraph::new();

        // add nodes
        for (node_id, node) in self.nodes() {
            let node_txt = match node {
                Node::Operator(operator) => operator.to_pretty_string(),
                Node::Handoff { .. } => HANDOFF_NODE_STR.to_string(),
            };
            g.nodes.insert(node_id, node_txt);
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

        let subgraph_handoffs = self.helper_collect_subgraph_handoffs();

        // add barrier_handoffs, i.e. handoffs that are *not* in the subgraph_recv_handoffs and
        // subgraph_send_handoffs for the same subgraph
        for sg in self.subgraph_nodes.keys() {
            let (recvs, sends) = &subgraph_handoffs[sg];
            for recv in recvs {
                if !sends.contains(recv) {
                    g.barrier_handoffs.insert(*recv, true);
                }
            }
        }

        // add subgraphs
        g.subgraph_nodes = self.subgraph_nodes.clone();
        g.subgraph_stratum = self.subgraph_stratum.clone();
        g.subgraph_internal_handoffs = {
            let mut subgraph_internal_handoffs: SecondaryMap<GraphSubgraphId, Vec<GraphNodeId>> =
                SecondaryMap::new();
            // iterate through edges, find internal handoffs and their inbound/outbound edges
            for e in self.graph.edges() {
                let (src, dst) = e.1;
                if let Node::Handoff { .. } = self.nodes[src] {
                    // Should only be one inbound_src, since it's a handoff.
                    for inbound_src in self.graph.predecessor_vertices(src) {
                        // Found an inbound edge to this handoff. Check if it's in the same subgraph as dst
                        if let Some(inbound_src_subgraph) = self.node_subgraph.get(inbound_src) {
                            if let Some(dst_subgraph) = self.node_subgraph.get(dst) {
                                if inbound_src_subgraph == dst_subgraph {
                                    // Found an internal handoff
                                    if let Node::Handoff { .. } = self.nodes[src] {
                                        subgraph_internal_handoffs
                                            .entry(*inbound_src_subgraph)
                                            .unwrap()
                                            .or_insert(Vec::new())
                                            .push(src);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            subgraph_internal_handoffs
        };

        // add varnames (sort for determinism).
        let mut varnames_sorted = self
            .node_varnames
            .iter()
            .map(|(node_id, Varname(ident))| (node_id, ident))
            .collect::<Vec<_>>();
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

    /// Convert back into surface syntax.
    pub fn surface_syntax_string(&self) -> String {
        let mut string = String::new();
        self.write_surface_syntax(&mut string).unwrap();
        string
    }

    /// Convert back into surface syntax.
    pub fn write_surface_syntax(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (key, node) in self.nodes.iter() {
            match node {
                Node::Operator(op) => {
                    writeln!(write, "{:?} = {};", key.data(), op.to_token_stream())?;
                }
                Node::Handoff { .. } => unimplemented!("HANDOFF IN FLAT GRAPH."),
            }
        }
        writeln!(write)?;
        for (_e, (src_key, dst_key)) in self.graph.edges() {
            writeln!(write, "{:?} -> {:?};", src_key.data(), dst_key.data())?;
        }
        Ok(())
    }

    /// Convert into a [mermaid](https://mermaid-js.github.io/) graph. Ignores subgraphs.
    pub fn mermaid_string_flat(&self) -> String {
        let mut string = String::new();
        self.write_mermaid_flat(&mut string).unwrap();
        string
    }

    /// Convert into a [mermaid](https://mermaid-js.github.io/) graph. Ignores subgraphs.
    pub fn write_mermaid_flat(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(write, "flowchart TB")?;
        for (key, node) in self.nodes.iter() {
            match node {
                Node::Operator(operator) => writeln!(
                    write,
                    "    %% {span}\n    {id:?}[\"{row_col} <tt>{code}</tt>\"]",
                    span = PrettySpan(node.span()),
                    id = key.data(),
                    row_col = PrettyRowCol(node.span()),
                    code = operator
                        .to_token_stream()
                        .to_string()
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;")
                        .replace('"', "&quot;")
                        .replace('\n', "<br>"),
                ),
                Node::Handoff { .. } => {
                    writeln!(write, r#"    {:?}{{"{}"}}"#, key.data(), HANDOFF_NODE_STR)
                }
            }?;
        }
        writeln!(write)?;
        for (_e, (src_key, dst_key)) in self.graph.edges() {
            writeln!(write, "    {:?}-->{:?}", src_key.data(), dst_key.data())?;
        }
        Ok(())
    }
}
