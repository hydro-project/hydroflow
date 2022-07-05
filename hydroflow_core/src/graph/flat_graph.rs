use std::collections::HashMap;

use quote::ToTokens;
use slotmap::{Key, SecondaryMap, SlotMap};
use syn::punctuated::Pair;
use syn::spanned::Spanned;
use syn::{Ident, LitInt};

use crate::parse::{HfCode, HfStatement, Pipeline};
use crate::pretty_span::PrettySpan;

use super::partitioned_graph::PartitionedGraph;
use super::{EdgePort, EdgePortRef, Node, NodeId};

#[derive(Debug, Default)]
pub struct FlatGraph {
    pub(crate) nodes: SlotMap<NodeId, Node>,
    pub(crate) preds: SecondaryMap<NodeId, HashMap<LitInt, EdgePort>>,
    pub(crate) succs: SecondaryMap<NodeId, HashMap<LitInt, EdgePort>>,
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
                            let src_port = curr_arrow.src.map(|x| x.index).unwrap_or_else(|| {
                                LitInt::new(
                                    &*self.succs[out].len().to_string(),
                                    curr_arrow.arrow.span(),
                                )
                            });
                            let dst_port = curr_arrow.dst.map(|x| x.index).unwrap_or_else(|| {
                                LitInt::new(
                                    &*self.preds[inn].len().to_string(),
                                    curr_arrow.arrow.span(),
                                )
                            });

                            {
                                /// Helper to emit conflicts when a port is overwritten.
                                fn emit_conflict(inout: &str, old: &LitInt, new: &LitInt) {
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
        use std::ops::{Bound, RangeBounds};
        trait RangeTrait<T>
        where
            T: ?Sized,
        {
            fn start_bound(&self) -> Bound<&T>;
            fn end_bound(&self) -> Bound<&T>;
            fn contains(&self, item: &T) -> bool
            where
                T: PartialOrd<T>;
        }
        impl<R, T> RangeTrait<T> for R
        where
            R: RangeBounds<T>,
        {
            fn start_bound(&self) -> Bound<&T> {
                self.start_bound()
            }

            fn end_bound(&self) -> Bound<&T> {
                self.end_bound()
            }

            fn contains(&self, item: &T) -> bool
            where
                T: PartialOrd<T>,
            {
                self.contains(item)
            }
        }

        for (node_key, node) in self.nodes.iter() {
            match node {
                Node::Operator(operator) => {
                    let op_name = &*operator.path.to_token_stream().to_string();
                    let (inn_allowed, out_allowed): (
                        &dyn RangeTrait<usize>,
                        &dyn RangeTrait<usize>,
                    ) = match op_name {
                        "merge" => (&(2..), &(1..=1)),
                        "join" => (&(2..=2), &(1..=1)),
                        "tee" => (&(1..=1), &(2..)),
                        "map" | "dedup" => (&(1..=1), &(1..=1)),
                        "input" | "seed" => (&(0..=0), &(1..=1)),
                        "for_each" => (&(1..=1), &(0..=0)),
                        unknown => {
                            operator
                                .path
                                .span()
                                .unwrap()
                                .error(format!("Unknown operator `{}`", unknown))
                                .emit();
                            (&(..), &(..))
                        }
                    };

                    let inn_degree = self.preds[node_key].len();
                    if !inn_allowed.contains(&inn_degree) {
                        operator
                            .span()
                            .unwrap()
                            .error(format!(
                        "`{}` has invalid number of inputs: {}. Allowed is between {:?} and {:?}.",
                        op_name,
                        inn_degree,
                        inn_allowed.start_bound(),
                        inn_allowed.end_bound()
                    ))
                            .emit();
                    }

                    let out_degree = self.succs[node_key].len();
                    if !out_allowed.contains(&out_degree) {
                        operator
                            .span()
                            .unwrap()
                            .error(format!(
                        "`{}` has invalid number of outputs: {}. Allowed is between {:?} and {:?}.",
                        op_name,
                        out_degree,
                        out_allowed.start_bound(),
                        out_allowed.end_bound()
                    ))
                            .emit();
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
                    r#"    {}["{}"]"#,
                    key.data().as_ffi(),
                    operator
                        .to_token_stream()
                        .to_string()
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;")
                        .replace('"', "&quot;"),
                ),
                Node::Handoff => writeln!(write, r#"    {}{{"handoff"}}"#, key.data().as_ffi()),
            }?;
        }
        writeln!(write)?;
        for (src_key, _op) in self.nodes.iter() {
            for (_src_port, (dst_key, _dst_port)) in self.succs[src_key].iter() {
                writeln!(
                    write,
                    "    {}-->{}",
                    src_key.data().as_ffi(),
                    dst_key.data().as_ffi()
                )?;
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
