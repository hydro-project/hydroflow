// TODO(mingwei): better error/diagnostic handling. Maybe collect all diagnostics before emitting.

use std::borrow::Cow;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Span;
use quote::ToTokens;
use slotmap::{Key, SecondaryMap, SlotMap, SparseSecondaryMap};
use syn::spanned::Spanned;
use syn::Ident;

use crate::diagnostic::{Diagnostic, Level};
use crate::graph::ops::{PortListSpec, RangeTrait, OPERATORS};
use crate::parse::{HfCode, HfStatement, Operator, Pipeline};
use crate::pretty_span::{PrettyRowCol, PrettySpan};

use super::di_mul_graph::DiMulGraph;
use super::partitioned_graph::PartitionedGraph;
use super::{GraphEdgeId, GraphNodeId, Node, PortIndexValue};

#[derive(Debug, Default)]
pub struct FlatGraphBuilder {
    /// Spanned error/warning/etc diagnostics to emit.
    diagnostics: Vec<Diagnostic>,

    /// FlatGraph being built.
    flat_graph: FlatGraph,
    /// Variable names, used as [`HfStatement::Named`] are added.
    /// Value will be set to `Err(())` if the name references an illegal self-referential cycle.
    varname_ends: BTreeMap<Ident, Result<Ends, ()>>,
    /// Each (out -> inn) link inputted.
    links: Vec<Ends>,
}

impl FlatGraphBuilder {
    /// Create a new empty graph builder.
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_hfcode(input: HfCode) -> Self {
        input.into()
    }

    /// Build into a [`FlatGraph`].
    ///
    /// Emits any diagnostics more severe than `min_diagnostic_level`.
    ///
    /// Returns `Err(FlatGraph)` if there are any errors. Returns `Ok(FlatGraph)` if there are no
    /// errors.
    pub fn build(mut self, min_diagnostic_level: Level) -> Result<FlatGraph, FlatGraph> {
        self.connect_operator_links();
        self.check_operator_errors();

        self.diagnostics
            .iter()
            .filter(|&diag| diag.level <= min_diagnostic_level)
            .for_each(Diagnostic::emit);
        if self.diagnostics.iter().any(Diagnostic::is_error) {
            Err(self.flat_graph)
        } else {
            Ok(self.flat_graph)
        }
    }

    /// Add a single [`HfStatement`] line to this `FlatGraph`.
    pub fn add_statement(&mut self, stmt: HfStatement) {
        let stmt_span = stmt.span();
        match stmt {
            HfStatement::Named(named) => {
                let ends = self.add_pipeline(named.pipeline, Some(&named.name));
                match self.varname_ends.entry(named.name) {
                    Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert(Ok(ends));
                    }
                    Entry::Occupied(occupied_entry) => {
                        let prev_conflict = occupied_entry.key();
                        self.diagnostics.push(Diagnostic::spanned(
                            stmt_span,
                            Level::Error,
                            format!(
                                "Name assignment to `{}` conflicts with existing assignment: {} (1/2)",
                                prev_conflict,
                                PrettySpan(prev_conflict.span())
                            ),
                        ));
                        self.diagnostics.push(Diagnostic::spanned(
                            prev_conflict.span(),
                            Level::Error,
                            format!(
                                "Existing assignment to `{}` conflicts with later assignment: {} (2/2)",
                                prev_conflict,
                                PrettySpan(stmt_span),
                            ),
                        ));
                    }
                }
            }
            HfStatement::Pipeline(pipeline) => {
                self.add_pipeline(pipeline, None);
            }
        }
    }

    /// Helper: Add a pipeline, i.e. `a -> b -> c`. Return the input and output ends for it.
    fn add_pipeline(&mut self, pipeline: Pipeline, current_varname: Option<&Ident>) -> Ends {
        match pipeline {
            Pipeline::Paren(ported_pipeline_paren) => {
                let (inn_port, pipeline_paren, out_port) =
                    PortIndexValue::from_ported(ported_pipeline_paren);
                let og_ends = self.add_pipeline(*pipeline_paren.pipeline, current_varname);
                Self::helper_combine_ends(&mut self.diagnostics, og_ends, inn_port, out_port)
            }
            Pipeline::Name(pipeline_name) => {
                let (inn_port, ident, out_port) = PortIndexValue::from_ported(pipeline_name);
                // We could lookup non-forward references immediately, but easier to just have one
                // consistent code path. -mingwei
                Ends {
                    inn: Some((inn_port, GraphDet::Undetermined(ident.clone()))),
                    out: Some((out_port, GraphDet::Undetermined(ident))),
                }
            }
            Pipeline::Link(pipeline_link) => {
                // Add the nested LHS and RHS of this link.
                let lhs_ends = self.add_pipeline(*pipeline_link.lhs, current_varname);
                let rhs_ends = self.add_pipeline(*pipeline_link.rhs, current_varname);

                // Outer (first and last) ends.
                let outer_ends = Ends {
                    inn: lhs_ends.inn,
                    out: rhs_ends.out,
                };
                // Inner (link) ends.
                let link_ends = Ends {
                    out: lhs_ends.out,
                    inn: rhs_ends.inn,
                };
                self.links.push(link_ends);
                outer_ends
            }
            Pipeline::Operator(operator) => {
                let op_span = operator.span();
                let key = self.flat_graph.nodes.insert(Node::Operator(operator));
                if let Some(current_varname) = current_varname {
                    self.flat_graph
                        .node_varnames
                        .insert(key, current_varname.clone());
                }
                Ends {
                    inn: Some((PortIndexValue::Elided(op_span), GraphDet::Determined(key))),
                    out: Some((PortIndexValue::Elided(op_span), GraphDet::Determined(key))),
                }
            }
        }
    }

    /// Connects operator links as a final building step. Processes all the links stored in
    /// `self.links` and actually puts them into the graph.
    fn connect_operator_links(&mut self) {
        for Ends { out, inn } in std::mem::take(&mut self.links) {
            let out_opt = self.helper_resolve_name(out, false);
            let inn_opt = self.helper_resolve_name(inn, true);
            // `None` already have errors in `self.diagnostics`.
            if let (Some((out_port, out_node)), Some((inn_port, inn_node))) = (out_opt, inn_opt) {
                self.connect_operators(out_port, out_node, inn_port, inn_node);
            }
        }
    }
    /// Recursively resolve a variable name. For handling forward (and backward) name references
    /// after all names have been assigned.
    /// Returns `None` if the name is not resolvable, either because it was never assigned or
    /// because it contains a self-referential cycle.
    fn helper_resolve_name(
        &mut self,
        mut port_det: Option<(PortIndexValue, GraphDet)>,
        is_in: bool,
    ) -> Option<(PortIndexValue, GraphNodeId)> {
        const BACKUP_RECURSION_LIMIT: usize = 1024;

        let mut names = Vec::new();
        for _ in 0..BACKUP_RECURSION_LIMIT {
            match port_det? {
                (port, GraphDet::Determined(node_id)) => {
                    return Some((port, node_id));
                }
                (port, GraphDet::Undetermined(ident)) => {
                    let Some(name_ends_result) = self.varname_ends.get(&ident) else {
                        self.diagnostics.push(Diagnostic::spanned(
                            ident.span(),
                            Level::Error,
                            format!("Cannot find name `{}`; name was never assigned.", ident),
                        ));
                        return None;
                    };
                    // Check for a self-referential cycle.
                    let cycle_found = names.contains(&ident);
                    if !cycle_found {
                        names.push(ident);
                    };
                    if cycle_found || name_ends_result.is_err() {
                        let len = names.len();
                        for (i, name) in names.into_iter().enumerate() {
                            self.diagnostics.push(Diagnostic::spanned(
                                name.span(),
                                Level::Error,
                                format!(
                                    "Name `{}` forms or references an illegal self-referential cycle ({}/{}).",
                                    name,
                                    i + 1,
                                    len
                                ),
                            ));
                            // Set value as `Err(())` to trigger `name_ends_result.is_err()`
                            // diagnostics above if the name is referenced in the future.
                            self.varname_ends.insert(name, Err(()));
                        }
                        return None;
                    }

                    // No self-cycle.
                    let name_ends = name_ends_result.as_ref().unwrap();
                    let prev = if is_in {
                        &name_ends.inn
                    } else {
                        &name_ends.out
                    };
                    port_det = Self::helper_combine_end(
                        &mut self.diagnostics,
                        prev.clone(),
                        port,
                        if is_in { "input" } else { "output" },
                    );
                }
            }
        }
        self.diagnostics.push(Diagnostic::spanned(
            Span::call_site(),
            Level::Error,
            format!(
                "Reached the recursion limit {} while resolving names. This is either a hydroflow bug or you have an absurdly long chain of names: `{}`.",
                BACKUP_RECURSION_LIMIT,
                names.iter().map(ToString::to_string).collect::<Vec<_>>().join("` -> `"),
            )
        ));
        None
    }
    /// Connect two operators on the given port indexes.
    fn connect_operators(
        &mut self,
        src_port: PortIndexValue,
        src: GraphNodeId,
        dst_port: PortIndexValue,
        dst: GraphNodeId,
    ) {
        {
            /// Helper to emit conflicts when a port is used twice.
            fn emit_conflict(
                inout: &str,
                old: &PortIndexValue,
                new: &PortIndexValue,
                diagnostics: &mut Vec<Diagnostic>,
            ) {
                // TODO(mingwei): Use `MultiSpan` once `proc_macro2` supports it.
                diagnostics.push(Diagnostic::spanned(
                    old.span(),
                    Level::Error,
                    format!(
                        "{} connection conflicts with below ({}) (1/2)",
                        inout,
                        PrettySpan(new.span()),
                    ),
                ));
                diagnostics.push(Diagnostic::spanned(
                    new.span(),
                    Level::Error,
                    format!(
                        "{} connection conflicts with above ({}) (2/2)",
                        inout,
                        PrettySpan(old.span()),
                    ),
                ));
            }

            // Handle src's successor port conflicts:
            if src_port.is_specified() {
                for conflicting_edge in self
                    .flat_graph
                    .graph
                    .successor_edges(src)
                    .filter(|&e| self.flat_graph.ports[e].0 == src_port)
                {
                    emit_conflict(
                        "Output",
                        &self.flat_graph.ports[conflicting_edge].0,
                        &src_port,
                        &mut self.diagnostics,
                    );
                }
            }

            // Handle dst's predecessor port conflicts:
            if dst_port.is_specified() {
                for conflicting_edge in self
                    .flat_graph
                    .graph
                    .predecessor_edges(dst)
                    .filter(|&e| self.flat_graph.ports[e].1 == dst_port)
                {
                    emit_conflict(
                        "Input",
                        &self.flat_graph.ports[conflicting_edge].1,
                        &dst_port,
                        &mut self.diagnostics,
                    );
                }
            }
        }

        let e = self.flat_graph.graph.insert_edge(src, dst);
        self.flat_graph.ports.insert(e, (src_port, dst_port));
    }

    /// Validates that operators have valid number of inputs, outputs, & arguments.
    /// Adds errors (and warnings) to `self.diagnostics`.
    /// TODO(mingwei): Clean this up, make it do more than just arity? Do no overlapping edge ports.
    fn check_operator_errors(&mut self) {
        for (node_key, node) in self.flat_graph.nodes.iter() {
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

                            let inn_degree = self.flat_graph.graph.degree_in(node_key);
                            let _ = emit_arity_error(
                                operator,
                                true,
                                true,
                                inn_degree,
                                op_constraints.hard_range_inn,
                                &mut self.diagnostics,
                            ) || emit_arity_error(
                                operator,
                                true,
                                false,
                                inn_degree,
                                op_constraints.soft_range_inn,
                                &mut self.diagnostics,
                            );

                            let out_degree = self.flat_graph.graph.degree_out(node_key);
                            let _ = emit_arity_error(
                                operator,
                                false,
                                true,
                                out_degree,
                                op_constraints.hard_range_out,
                                &mut self.diagnostics,
                            ) || emit_arity_error(
                                operator,
                                false,
                                false,
                                out_degree,
                                op_constraints.soft_range_out,
                                &mut self.diagnostics,
                            );

                            fn emit_port_error<'a>(
                                operator_span: Span,
                                expected_ports_fn: Option<&dyn Fn() -> PortListSpec>,
                                actual_ports_iter: impl Iterator<Item = &'a PortIndexValue>,
                                input_output: &'static str,
                                diagnostics: &mut Vec<Diagnostic>,
                            ) {
                                let Some(expected_ports_fn) = expected_ports_fn else {
                                    return;
                                };
                                let PortListSpec::Fixed(expected_ports) = (expected_ports_fn)() else {
                                    // Separate check inside of `demux` special case.
                                    return;
                                };
                                let expected_ports: Vec<_> = expected_ports.into_iter().collect();

                                // Reject unexpected ports.
                                let ports: BTreeSet<_> = actual_ports_iter
                                    .inspect(|port_index_value| {
                                        if !expected_ports.iter().any(|port_index| {
                                            port_index_value == &&port_index.clone().into()
                                        }) {
                                            diagnostics.push(Diagnostic::spanned(
                                                port_index_value.span(),
                                                Level::Error,
                                                format!(
                                                    "Unexpected {} port. Expected one of: `{}`",
                                                    input_output,
                                                    expected_ports
                                                        .iter()
                                                        .map(|port| Cow::Owned(
                                                            port.to_token_stream().to_string()
                                                        ))
                                                        .intersperse(Cow::Borrowed("`, `"))
                                                        .collect::<String>()
                                                ),
                                            ))
                                        }
                                    })
                                    .collect();

                                // List missing ports.
                                let missing: Vec<_> = expected_ports
                                    .into_iter()
                                    .filter_map(|expected_port| {
                                        let tokens = expected_port.to_token_stream();
                                        if !ports.contains(&&expected_port.into()) {
                                            Some(tokens)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                if !missing.is_empty() {
                                    diagnostics.push(Diagnostic::spanned(
                                        operator_span,
                                        Level::Error,
                                        format!(
                                            "Missing expected {} port(s): `{}`.",
                                            input_output,
                                            missing
                                                .into_iter()
                                                .map(|port| Cow::Owned(
                                                    port.to_token_stream().to_string()
                                                ))
                                                .intersperse(Cow::Borrowed("`, `"))
                                                .collect::<String>()
                                        ),
                                    ));
                                }
                            }

                            emit_port_error(
                                operator.span(),
                                op_constraints.ports_inn,
                                self.flat_graph
                                    .graph
                                    .predecessor_edges(node_key)
                                    .map(|edge_id| &self.flat_graph.ports[edge_id].1),
                                "input",
                                &mut self.diagnostics,
                            );
                            emit_port_error(
                                operator.span(),
                                op_constraints.ports_out,
                                self.flat_graph
                                    .graph
                                    .successor_edges(node_key)
                                    .map(|edge_id| &self.flat_graph.ports[edge_id].0),
                                "output",
                                &mut self.diagnostics,
                            );
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
                Node::Handoff { .. } => todo!("Node::Handoff"),
            }
        }
    }

    /// Helper function.
    /// Combine the port indexing information for indexing wrapped around a name.
    /// Because the name may already have indexing, this may introduce double indexing (i.e. `[0][0]my_var[0][0]`)
    /// which would be an error.
    fn helper_combine_ends(
        diagnostics: &mut Vec<Diagnostic>,
        og_ends: Ends,
        inn_port: PortIndexValue,
        out_port: PortIndexValue,
    ) -> Ends {
        Ends {
            inn: Self::helper_combine_end(diagnostics, og_ends.inn, inn_port, "input"),
            out: Self::helper_combine_end(diagnostics, og_ends.out, out_port, "output"),
        }
    }

    /// Helper function.
    /// Combine the port indexing info for one input or output.
    fn helper_combine_end(
        diagnostics: &mut Vec<Diagnostic>,
        og: Option<(PortIndexValue, GraphDet)>,
        other: PortIndexValue,
        input_output: &'static str,
    ) -> Option<(PortIndexValue, GraphDet)> {
        // TODO(mingwei): minification pass over this code?

        let other_span = other.span();

        let (og_port, og_node) = og?;
        match og_port.combine(other) {
            Ok(combined_port) => Some((combined_port, og_node)),
            Err(og_port) => {
                // TODO(mingwei): Use `MultiSpan` once `proc_macro2` supports it.
                diagnostics.push(Diagnostic::spanned(
                    og_port.span(),
                    Level::Error,
                    format!(
                        "Indexing on {} is overwritten below ({}) (1/2).",
                        input_output,
                        PrettySpan(other_span),
                    ),
                ));
                diagnostics.push(Diagnostic::spanned(
                    other_span,
                    Level::Error,
                    format!(
                        "Cannot index on already-indexed {}, previously indexed above ({}) (2/2).",
                        input_output,
                        PrettySpan(og_port.span()),
                    ),
                ));
                // When errored, just use original and ignore OTHER port to minimize
                // noisy/extra diagnostics.
                Some((og_port, og_node))
            }
        }
    }
}

impl From<HfCode> for FlatGraphBuilder {
    fn from(input: HfCode) -> Self {
        let mut builder = Self::default();
        for stmt in input.statements {
            builder.add_statement(stmt);
        }
        builder
    }
}

/// A graph representing a hydroflow dataflow graph before subgraph partitioning, stratification, and handoff insertion.
/// I.e. the graph is a simple "flat" without any subgraph heirarchy.
///
/// May optionally contain handoffs, but in this stage these are transparent and treated like an identity operator.
///
/// Use `Self::into_partitioned_graph()` to convert into a subgraph-partitioned & stratified graph.
#[derive(Debug, Default)]
pub struct FlatGraph {
    /// Each node (operator or handoff).
    pub(super) nodes: SlotMap<GraphNodeId, Node>,
    /// Graph
    pub(super) graph: DiMulGraph<GraphNodeId, GraphEdgeId>,
    /// Input and output port for each edge.
    pub(super) ports: SecondaryMap<GraphEdgeId, (PortIndexValue, PortIndexValue)>,

    /// What variable name each graph node belongs to (if any).
    pub(super) node_varnames: SparseSecondaryMap<GraphNodeId, Ident>,
}

impl FlatGraph {
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
                Node::Handoff { .. } => writeln!(write, r#"    {:?}{{"handoff"}}"#, key.data()),
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

#[derive(Clone, Debug)]
struct Ends {
    inn: Option<(PortIndexValue, GraphDet)>,
    out: Option<(PortIndexValue, GraphDet)>,
}

#[derive(Clone, Debug)]
enum GraphDet {
    Determined(GraphNodeId),
    Undetermined(Ident),
}
