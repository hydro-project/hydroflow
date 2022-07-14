use std::collections::HashMap;

use quote::ToTokens;
use slotmap::{Key, SecondaryMap, SlotMap};
use syn::punctuated::Pair;
use syn::spanned::Spanned;
use syn::Ident;

use crate::graph::ops::{RangeTrait, OPERATORS};
use crate::parse::{HfCode, HfStatement, IndexInt, Operator, Pipeline};
use crate::pretty_span::PrettySpan;

use super::partitioned_graph::PartitionedGraph;
use super::{EdgePortRef, Node, NodeId, OutboundEdges};

#[derive(Debug, Default)]
pub struct FlatGraph {
    pub(crate) nodes: SlotMap<NodeId, Node>,
    pub(crate) preds: SecondaryMap<NodeId, OutboundEdges>,
    pub(crate) succs: SecondaryMap<NodeId, OutboundEdges>,
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

    fn add_statement(&mut self, stmt: HfStatement) {
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
            Pipeline::Chain(chain_pipeline) => {
                // Handle chain pipelines as follows:
                let output = chain_pipeline
                    .elems
                    .into_pairs()
                    .map(Pair::into_tuple)
                    // 1. Resolve all the nested pipelines in first stage (collect into Vec before continuing, for ownership).
                    .map(|(pipeline, arrow)| (self.add_pipeline(pipeline), arrow))
                    .collect::<Vec<_>>()
                    .into_iter()
                    // 2. Iterate each element in pairs via `.reduce()` and combine them into the next pipeline.
                    // Essentially, treats the arrows as a left-associative binary operation (not that the direction really matters).
                    // `curr_ports: Ports` tracks the current input/output operators/ports in the graph.
                    .reduce(|(curr_ports, curr_arrow), (next_ports, next_arrow)| {
                        let curr_arrow =
                            curr_arrow.expect("Cannot have missing intermediate arrow");

                        if let (Some(out), Some(inn)) = (curr_ports.out, next_ports.inn) {
                            let src_port =
                                curr_arrow.src.map(|x| x.index).unwrap_or_else(|| IndexInt {
                                    value: self.succs[out].len(),
                                    span: curr_arrow.arrow.span(),
                                });
                            let dst_port =
                                curr_arrow.dst.map(|x| x.index).unwrap_or_else(|| IndexInt {
                                    value: self.preds[inn].len(),
                                    span: curr_arrow.arrow.span(),
                                });

                            {
                                /// Helper to emit conflicts when a port is overwritten.
                                fn emit_conflict(inout: &str, old: &IndexInt, new: &IndexInt) {
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

                                // Clone, one for `succs` and one for `preds`.
                                let (src_a, src_b) = (src_port.clone(), src_port);
                                let (dst_a, dst_b) = (dst_port.clone(), dst_port);

                                if let Some((old_a, _)) = self.succs[out].remove_entry(&src_a) {
                                    emit_conflict("Output", &old_a, &src_a);
                                }
                                self.succs[out].insert(src_a, (inn, dst_a));

                                if let Some((old_b, _)) = self.preds[inn].remove_entry(&dst_b) {
                                    emit_conflict("Input", &old_b, &dst_b);
                                }
                                self.preds[inn].insert(dst_b, (out, src_b));
                            }
                        }

                        let ports = Ports {
                            inn: curr_ports.inn,
                            out: next_ports.out,
                        };
                        (ports, next_arrow)
                    });

                output.map(|(ports, _arrow)| ports).unwrap_or(Ports {
                    inn: None,
                    out: None,
                })
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

    /// Validates that operators have valid number of inputs and outputs.
    /// (Emits error messages on span).
    /// TODO(mingwei): Clean this up, make it do more than just arity.
    pub fn validate_operators(&self) {
        for (node_key, node) in self.nodes.iter() {
            match node {
                Node::Operator(operator) => {
                    let op_name = &*operator.name_string();
                    match OPERATORS.iter().find(|&op| op_name == op.name) {
                        Some(op_constraints) => {
                            fn emit_arity_error(
                                operator: &Operator,
                                is_in: bool,
                                is_hard: bool,
                                degree: usize,
                                range: &dyn RangeTrait<usize>,
                            ) {
                                let op_name = &*operator.name_string();
                                let message = format!(
                                    "`{}` {} have {} {}, actually has {}.",
                                    op_name,
                                    if is_hard { "must" } else { "should" },
                                    range.human_string(),
                                    if is_in { "inputs" } else { "outputs" },
                                    degree,
                                );
                                if !range.contains(&degree) {
                                    if is_hard {
                                        operator.span().unwrap().error(message).emit();
                                    } else {
                                        operator.span().unwrap().warning(message).emit();
                                    }
                                }
                            }

                            let inn_degree = self.preds[node_key].len();
                            emit_arity_error(
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
                            emit_arity_error(
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
                        }
                    }
                }
                Node::Handoff => todo!("Node::Handoff"),
            }
        }
    }

    pub fn edges(&self) -> impl '_ + Iterator<Item = (EdgePortRef, EdgePortRef)> {
        super::iter_edges(&self.succs)
    }

    pub fn mermaid_string(&self) -> String {
        let mut string = String::new();
        self.write_mermaid(&mut string).unwrap();
        string
    }

    #[allow(dead_code)]
    pub fn write_mermaid(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(write, "flowchart TB")?;
        for (key, node) in self.nodes.iter() {
            match node {
                Node::Operator(operator) => writeln!(
                    write,
                    r#"    {:?}["{}"]"#,
                    key.data(),
                    operator
                        .to_token_stream()
                        .to_string()
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;")
                        .replace('"', "&quot;"),
                ),
                Node::Handoff => writeln!(write, r#"    {:?}{{"handoff"}}"#, key.data()),
            }?;
        }
        writeln!(write)?;
        for (src_key, _op) in self.nodes.iter() {
            for (_src_port, (dst_key, _dst_port)) in self.succs[src_key].iter() {
                writeln!(write, "    {:?}-->{:?}", src_key.data(), dst_key.data())?;
            }
        }
        Ok(())
    }

    pub fn into_partitioned_graph(self) -> PartitionedGraph {
        self.into()
    }
}

#[derive(Clone, Copy, Debug)]
struct Ports {
    inn: Option<NodeId>,
    out: Option<NodeId>,
}
