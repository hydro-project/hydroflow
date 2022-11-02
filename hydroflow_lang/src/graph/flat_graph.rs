use std::collections::{BTreeMap, BTreeSet};

use quote::ToTokens;
use slotmap::{Key, SecondaryMap, SlotMap};
use syn::spanned::Spanned;
use syn::Ident;

use crate::diagnostic::{Diagnostic, Level};
use crate::graph::ops::{RangeTrait, OPERATORS};
use crate::parse::{HfCode, HfStatement, Operator, Pipeline};
use crate::pretty_span::{PrettyRowCol, PrettySpan};

use super::di_mul_graph::DiMulGraph;
use super::partitioned_graph::PartitionedGraph;
use super::{GraphEdgeId, GraphNodeId, Node, PortIndexValue};

/// A graph representing a hydroflow dataflow graph before subgraph partitioning, stratification, and handoff insertion.
/// I.e. the graph is a simple "flat" without any subgraph heirarchy.
///
/// May optionally contain handoffs, but in this stage these are transparent and treated like an identity operator.
///
/// Use `Self::into_partitioned_graph()` to convert into a subgraph-partitioned & stratified graph.
#[derive(Debug, Default)]
pub struct FlatGraph {
    /// Each node (operator or handoff).
    pub(crate) nodes: SlotMap<GraphNodeId, Node>,
    /// Graph
    pub(crate) graph: DiMulGraph<GraphNodeId, GraphEdgeId>,
    /// Input and output port for each edge.
    pub(crate) indices: SecondaryMap<GraphEdgeId, (PortIndexValue, PortIndexValue)>,
    /// Spanned error/warning/etc diagnostics to emit.
    pub(crate) diagnostics: Vec<Diagnostic>,

    /// Variable names, used as [`HfStatement::Named`] are added.
    names: BTreeMap<Ident, Ports>,
}

impl FlatGraph {
    /// Creates a new `FlatGraph` instance based on the [`HfCode`] AST.
    ///
    /// TODO(mingwei): better error/diagnostic handling. Maybe collect all diagnostics before emitting.
    pub fn from_hfcode(input: HfCode) -> Self {
        let mut graph = Self::default();

        for stmt in input.statements {
            graph.add_statement(stmt);
        }

        graph.check_operator_errors();
        graph
    }

    /// Add a single [`HfStatement`] line to this `FlatGraph`.
    pub fn add_statement(&mut self, stmt: HfStatement) {
        match stmt {
            HfStatement::Named(named) => {
                let ports = self.add_pipeline(named.pipeline);
                self.names.insert(named.name, ports);
            }
            HfStatement::Pipeline(pipeline) => {
                self.add_pipeline(pipeline);
            }
        }
    }

    /// Helper: Add a pipeline, i.e. `a -> b -> c`. Return the input and output port for it.
    fn add_pipeline(&mut self, pipeline: Pipeline) -> Ports {
        match pipeline {
            Pipeline::Paren(pipeline_paren) => self.add_pipeline(*pipeline_paren.pipeline),
            Pipeline::Link(pipeline_link) => {
                // Add the nested LHS and RHS of this link.
                let lhs_ports = self.add_pipeline(*pipeline_link.lhs);
                let connector = pipeline_link.connector;
                let rhs_ports = self.add_pipeline(*pipeline_link.rhs);

                if let (Some(src), Some(dst)) = (lhs_ports.out, rhs_ports.inn) {
                    let (src_port, dst_port) = PortIndexValue::from_arrow_connector(connector);

                    {
                        /// Helper to emit conflicts when a port is used twice.
                        fn emit_conflict(
                            inout: &str,
                            old: &PortIndexValue,
                            new: &PortIndexValue,
                            diagnostics: &mut Vec<Diagnostic>,
                        ) {
                            diagnostics.push(Diagnostic::spanned(
                                old.span(),
                                Level::Error,
                                format!(
                                    "{} connection conflicts with below ({})",
                                    inout,
                                    PrettySpan(new.span()),
                                ),
                            ));
                            diagnostics.push(Diagnostic::spanned(
                                new.span(),
                                Level::Error,
                                format!(
                                    "{} connection conflicts with above ({})",
                                    inout,
                                    PrettySpan(old.span()),
                                ),
                            ));
                        }

                        // Handle src's successor port conflicts:
                        if src_port.is_specified() {
                            for conflicting_edge in self
                                .graph
                                .successor_edges(src)
                                .filter(|&e| self.indices[e].0 == src_port)
                            {
                                emit_conflict(
                                    "Output",
                                    &self.indices[conflicting_edge].0,
                                    &src_port,
                                    &mut self.diagnostics,
                                );
                            }
                        }

                        // Handle dst's predecessor port conflicts:
                        if dst_port.is_specified() {
                            for conflicting_edge in self
                                .graph
                                .predecessor_edges(dst)
                                .filter(|&e| self.indices[e].1 == dst_port)
                            {
                                emit_conflict(
                                    "Input",
                                    &self.indices[conflicting_edge].1,
                                    &dst_port,
                                    &mut self.diagnostics,
                                );
                            }
                        }
                    }

                    let e = self.graph.insert_edge(src, dst);
                    self.indices.insert(e, (src_port, dst_port));
                }

                Ports {
                    inn: lhs_ports.inn,
                    out: rhs_ports.out,
                }
            }
            Pipeline::Name(ident) => self.names.get(&ident).copied().unwrap_or_else(|| {
                self.diagnostics.push(Diagnostic::spanned(
                    ident.span(),
                    Level::Error,
                    format!("Cannot find name `{}`", ident),
                ));
                Ports {
                    inn: None,
                    out: None,
                }
            }),
            Pipeline::Operator(operator) => {
                let key = self.nodes.insert(Node::Operator(operator));
                Ports {
                    inn: Some(key),
                    out: Some(key),
                }
            }
        }
    }

    /// Validates that operators have valid number of inputs, outputs, & arguments.
    /// Adds errors (and warnings) to `self.diagnostics`.
    /// TODO(mingwei): Clean this up, make it do more than just arity? Do no overlapping edge ports.
    fn check_operator_errors(&mut self) {
        for (node_key, node) in self.nodes.iter() {
            match node {
                Node::Operator(operator) => {
                    let op_name = &*operator.name_string();
                    match OPERATORS.iter().find(|&op| op_name == op.name) {
                        Some(op_constraints) => {
                            // Check numer of args
                            if op_constraints.num_args != operator.args.len() {
                                self.diagnostics.push(Diagnostic::spanned(
                                    operator.span(),
                                    Level::Error,
                                    format!(
                                        "expected {} argument(s), found {}",
                                        op_constraints.num_args,
                                        operator.args.len()
                                    ),
                                ));
                            }

                            // Check input/output (port) arity
                            /// Returns true if an error was found.
                            fn emit_arity_error(
                                operator: &Operator,
                                is_in: bool,
                                is_hard: bool,
                                degree: usize,
                                range: &dyn RangeTrait<usize>,
                                diagnostics: &mut Vec<Diagnostic>,
                            ) -> bool {
                                let op_name = &*operator.name_string();
                                let message = format!(
                                    "`{}` {} have {} {}, actually has {}.",
                                    op_name,
                                    if is_hard { "must" } else { "should" },
                                    range.human_string(),
                                    if is_in { "input(s)" } else { "output(s)" },
                                    degree,
                                );
                                let out_of_range = !range.contains(&degree);
                                if out_of_range {
                                    diagnostics.push(Diagnostic::spanned(
                                        operator.span(),
                                        if is_hard {
                                            Level::Error
                                        } else {
                                            Level::Warning
                                        },
                                        message,
                                    ));
                                }
                                out_of_range
                            }

                            let inn_degree = self.graph.degree_in(node_key);
                            emit_arity_error(
                                operator,
                                true,
                                true,
                                inn_degree,
                                op_constraints.hard_range_inn,
                                &mut self.diagnostics,
                            );
                            emit_arity_error(
                                operator,
                                true,
                                false,
                                inn_degree,
                                op_constraints.soft_range_inn,
                                &mut self.diagnostics,
                            );

                            let out_degree = self.graph.degree_out(node_key);
                            emit_arity_error(
                                operator,
                                false,
                                true,
                                out_degree,
                                op_constraints.hard_range_out,
                                &mut self.diagnostics,
                            );
                            emit_arity_error(
                                operator,
                                false,
                                false,
                                out_degree,
                                op_constraints.soft_range_out,
                                &mut self.diagnostics,
                            );

                            // Check input port indexes/names
                            if let Some(ports_inn_fn) = op_constraints.ports_inn {
                                let inn_ports: BTreeSet<_> = self
                                    .graph
                                    .predecessor_edges(node_key)
                                    .map(|edge_id| &self.indices[edge_id].1)
                                    .collect();

                                for expected_port in (ports_inn_fn)() {
                                    let tokens = expected_port.to_token_stream();
                                    if !inn_ports.contains(&&expected_port.into()) {
                                        self.diagnostics.push(Diagnostic::spanned(
                                            operator.span(),
                                            Level::Error,
                                            format!("Missing expected input port `{}`.", tokens),
                                        ));
                                    }
                                }
                            }
                            if let Some(ports_out_fn) = op_constraints.ports_out {
                                let out_ports: BTreeSet<_> = self
                                    .graph
                                    .successor_edges(node_key)
                                    .map(|edge_id| &self.indices[edge_id].0)
                                    .collect();

                                for expected_port in (ports_out_fn)() {
                                    let tokens = expected_port.to_token_stream();
                                    if !out_ports.contains(&&expected_port.into()) {
                                        self.diagnostics.push(Diagnostic::spanned(
                                            operator.span(),
                                            Level::Error,
                                            format!("Missing expected output port `{}`.", tokens),
                                        ));
                                    }
                                }
                            }
                        }
                        None => {
                            self.diagnostics.push(Diagnostic::spanned(
                                operator.path.span(),
                                Level::Error,
                                format!("Unknown operator `{}`", op_name),
                            ));
                        }
                    }
                }
                Node::Handoff => todo!("Node::Handoff"),
            }
        }
    }

    /// Emits diagnostics, returns true if there are errors.
    pub fn emit_diagnostics(&self) -> bool {
        self.diagnostics.iter().for_each(Diagnostic::emit);
        self.diagnostics.iter().any(Diagnostic::is_error)
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
                Node::Handoff => unimplemented!("HANDOFF IN FLAT GRAPH."),
            }
        }
        writeln!(write)?;
        for (_e, (src_key, dst_key)) in self.graph.edges() {
            writeln!(write, "({:?}-->{:?});", src_key.data(), dst_key.data())?;
        }
        Ok(())
    }

    /// Convert into a [mermaid](https://mermaid-js.github.io/) graph.
    pub fn mermaid_string(&self) -> String {
        let mut string = String::new();
        self.write_mermaid(&mut string).unwrap();
        string
    }

    /// Convert into a [mermaid](https://mermaid-js.github.io/) graph.
    pub fn write_mermaid(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
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
                Node::Handoff => writeln!(write, r#"    {:?}{{"handoff"}}"#, key.data()),
            }?;
        }
        writeln!(write)?;
        for (_e, (src_key, dst_key)) in self.graph.edges() {
            writeln!(write, "    {:?}-->{:?}", src_key.data(), dst_key.data())?;
        }
        Ok(())
    }

    /// Run subgraph partitioning and stratification and convert this graph into a [`PartitionedGraph`].
    #[allow(clippy::result_unit_err)]
    pub fn into_partitioned_graph(self) -> Result<PartitionedGraph, Diagnostic> {
        self.try_into()
    }
}

#[derive(Clone, Copy, Debug)]
struct Ports {
    inn: Option<GraphNodeId>,
    out: Option<GraphNodeId>,
}
