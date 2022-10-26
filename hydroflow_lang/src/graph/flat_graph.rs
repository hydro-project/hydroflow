use std::collections::BTreeMap;

use quote::ToTokens;
use slotmap::{Key, SecondaryMap, SlotMap};
use syn::spanned::Spanned;
use syn::Ident;

use crate::graph::ops::{RangeTrait, OPERATORS};
use crate::parse::{HfCode, HfStatement, IndexInt, Operator, Pipeline};
use crate::pretty_span::{PrettyRowCol, PrettySpan};

use super::di_mul_graph::DiMulGraph;
use super::partitioned_graph::PartitionedGraph;
use super::{GraphEdgeId, GraphNodeId, Node};

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
    pub(crate) indices: SecondaryMap<GraphEdgeId, (IndexInt, IndexInt)>,

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

        graph
    }

    /// Add a single [`HfStatement`] line to this `FlatGraph`.
    pub fn add_statement(&mut self, stmt: HfStatement) {
        match stmt {
            HfStatement::Named(named) => {
                let ports = self.add_pipeline(named.pipeline);
                // if let Some((old_name, _)) = self.names.remove_entry(&named.name) {
                //     old_name.span().unwrap().warning(format!("`{}` is shadowed"))
                // }
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
                    let src_port = connector.src.map(|x| x.index).unwrap_or_else(|| IndexInt {
                        value: 0,
                        span: connector.arrow.span(),
                    });
                    let dst_port = connector.dst.map(|x| x.index).unwrap_or_else(|| IndexInt {
                        value: 0,
                        span: connector.arrow.span(),
                    });

                    {
                        /// Helper to emit conflicts when a port is overwritten.
                        fn emit_conflict(inout: &str, old: IndexInt, new: IndexInt) {
                            old.span()
                                .unwrap()
                                .error(format!(
                                    "{} connection conflicts with below ({})",
                                    inout,
                                    PrettySpan(new.span()),
                                ))
                                .emit();
                            new.span()
                                .unwrap()
                                .error(format!(
                                    "{} connection conflicts with above ({})",
                                    inout,
                                    PrettySpan(old.span()),
                                ))
                                .emit();
                        }

                        // Handle src's successor port conflicts:
                        for conflicting_edge in self
                            .graph
                            .successor_edges(src)
                            .filter(|&e| self.indices[e].0 == src_port)
                        {
                            emit_conflict("Output", self.indices[conflicting_edge].0, src_port);
                        }

                        // Handle dst's predecessor port conflicts:
                        for conflicting_edge in self
                            .graph
                            .predecessor_edges(dst)
                            .filter(|&e| self.indices[e].1 == dst_port)
                        {
                            emit_conflict("Input", self.indices[conflicting_edge].1, dst_port);
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
                ident
                    .span()
                    .unwrap()
                    .error(format!("Cannot find name `{ident}`"))
                    .emit();
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
    /// (Emits error messages on span).
    /// TODO(mingwei): Clean this up, make it do more than just arity? Do no overlapping edge ports.
    /// Returns `true` if errors were found.
    pub fn emit_operator_errors(&self) -> bool {
        let mut errored = false;
        for (node_key, node) in self.nodes.iter() {
            match node {
                Node::Operator(operator) => {
                    let op_name = &*operator.name_string();
                    match OPERATORS.iter().find(|&op| op_name == op.name) {
                        Some(op_constraints) => {
                            if op_constraints.num_args != operator.args.len() {
                                errored = true;

                                operator
                                    .span()
                                    .unwrap()
                                    .error(format!(
                                        "expected {} argument(s), found {}",
                                        op_constraints.num_args,
                                        operator.args.len()
                                    ))
                                    .emit();
                            }

                            /// Returns true if an error was found.
                            fn emit_arity_error(
                                operator: &Operator,
                                is_in: bool,
                                is_hard: bool,
                                degree: usize,
                                range: &dyn RangeTrait<usize>,
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
                                    if is_hard {
                                        operator.span().unwrap().error(message).emit();
                                    } else {
                                        operator.span().unwrap().warning(message).emit();
                                    }
                                }
                                out_of_range
                            }

                            let inn_degree = self.graph.degree_in(node_key);
                            errored |= emit_arity_error(
                                operator,
                                true,
                                true,
                                inn_degree,
                                op_constraints.hard_range_inn,
                            );
                            emit_arity_error(
                                operator,
                                true,
                                false,
                                inn_degree,
                                op_constraints.soft_range_inn,
                            );

                            let out_degree = self.graph.degree_out(node_key);
                            errored |= emit_arity_error(
                                operator,
                                false,
                                true,
                                out_degree,
                                op_constraints.hard_range_out,
                            );
                            emit_arity_error(
                                operator,
                                false,
                                false,
                                out_degree,
                                op_constraints.soft_range_out,
                            );
                        }
                        None => {
                            operator
                                .path
                                .span()
                                .unwrap()
                                .error(format!("Unknown operator `{}`", op_name))
                                .emit();
                            errored = true;
                        }
                    }
                }
                Node::Handoff => todo!("Node::Handoff"),
            }
        }
        errored
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
    pub fn into_partitioned_graph(self) -> Result<PartitionedGraph, ()> {
        self.try_into()
    }
}

#[derive(Clone, Copy, Debug)]
struct Ports {
    inn: Option<GraphNodeId>,
    out: Option<GraphNodeId>,
}
