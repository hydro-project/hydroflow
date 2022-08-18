use std::collections::HashMap;

use quote::ToTokens;
use slotmap::{Key, SecondaryMap, SlotMap};
use syn::spanned::Spanned;
use syn::Ident;

use crate::graph::ops::{RangeTrait, OPERATORS};
use crate::parse::{HfCode, HfStatement, IndexInt, Operator, Pipeline};
use crate::pretty_span::{PrettyRowCol, PrettySpan};

use super::partitioned_graph::PartitionedGraph;
use super::{EdgePortRef, GraphNodeId, Node, OutboundEdges};

#[derive(Debug, Default)]
pub struct FlatGraph {
    pub(crate) nodes: SlotMap<GraphNodeId, Node>,
    pub(crate) preds: SecondaryMap<GraphNodeId, OutboundEdges>,
    pub(crate) succs: SecondaryMap<GraphNodeId, OutboundEdges>,
    names: HashMap<Ident, Ports>,
}
impl FlatGraph {
    // TODO(mingwei): better error/diagnostic handling.
    pub fn from_hfcode(input: HfCode) -> Self {
        let mut graph = Self::default();

        for stmt in input.statements {
            graph.add_statement(stmt);
        }

        graph
    }

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

    fn add_pipeline(&mut self, pipeline: Pipeline) -> Ports {
        match pipeline {
            Pipeline::Paren(pipeline_paren) => self.add_pipeline(*pipeline_paren.pipeline),
            Pipeline::Link(pipeline_link) => {
                let lhs_ports = self.add_pipeline(*pipeline_link.lhs);
                let connector = pipeline_link.connector;
                let rhs_ports = self.add_pipeline(*pipeline_link.rhs);

                if let (Some(out), Some(inn)) = (lhs_ports.out, rhs_ports.inn) {
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

                        if let Some((old_a, _)) = self.succs[out].remove_entry(&src_port) {
                            emit_conflict("Output", old_a, src_port);
                        }
                        self.succs[out].insert(src_port, (inn, dst_port));

                        if let Some((old_b, _)) = self.preds[inn].remove_entry(&dst_port) {
                            emit_conflict("Input", old_b, dst_port);
                        }
                        self.preds[inn].insert(dst_port, (out, src_port));
                    }
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
                    .error(format!("Cannot find name `{}`", ident))
                    .emit();
                Ports {
                    inn: None,
                    out: None,
                }
            }),
            Pipeline::Operator(operator) => {
                let key = self.nodes.insert(Node::Operator(operator));
                self.preds.insert(key, Default::default());
                self.succs.insert(key, Default::default());
                Ports {
                    inn: Some(key),
                    out: Some(key),
                }
            }
        }
    }

    /// Validates that operators have valid number of inputs, outputs, and arguments.
    /// (Emits error messages on span).
    /// TODO(mingwei): Clean this up, make it do more than just arity?
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

                            let inn_degree = self.preds[node_key].len();
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

                            let out_degree = self.succs[node_key].len();
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

    pub fn edges(&self) -> impl '_ + Iterator<Item = (EdgePortRef, EdgePortRef)> {
        super::iter_edges(&self.succs)
    }

    pub fn surface_syntax_string(&self) -> String {
        let mut string = String::new();
        self.write_surface_syntax(&mut string).unwrap();
        string
    }

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
        for ((src_key, _src_port), (dst_key, _dst_port)) in super::iter_edges(&self.succs) {
            writeln!(write, "({:?}-->{:?});", src_key.data(), dst_key.data())?;
        }
        Ok(())
    }

    pub fn mermaid_string(&self) -> String {
        let mut string = String::new();
        self.write_mermaid(&mut string).unwrap();
        string
    }

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
        for ((src_key, _src_port), (dst_key, _dst_port)) in super::iter_edges(&self.succs) {
            writeln!(write, "    {:?}-->{:?}", src_key.data(), dst_key.data())?;
        }
        Ok(())
    }

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
