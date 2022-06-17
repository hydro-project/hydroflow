#![feature(proc_macro_diagnostic, proc_macro_span)]

use std::collections::HashMap;

use proc_macro2::{Literal, Span};
use quote::{quote, ToTokens};
use slotmap::{DefaultKey, Key, SlotMap};
use syn::punctuated::Pair;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Ident, LitInt};

mod parse;
use parse::{HfCode, HfStatement, Operator, Pipeline};

#[proc_macro]
pub fn hydroflow_parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as HfCode);
    // // input.into_token_stream().into()

    let graph = Graph::from_hfcode(input).unwrap(/* TODO(mingwei) */);

    // let debug = format!("{:#?}", graph);
    // let mut debug = String::new();
    // graph.write_graph(&mut debug).unwrap();
    let debug = graph.mermaid_string();

    let lit = Literal::string(&*debug);

    quote! { println!("{}", #lit); }.into()
}

#[derive(Debug, Default)]
struct Graph {
    operators: SlotMap<DefaultKey, OpInfo>,
    names: HashMap<Ident, Ports>,
}
impl Graph {
    pub fn from_hfcode(input: HfCode) -> Result<Self, ()> {
        let mut graph = Self::default();

        for stmt in input.statements {
            graph.add_statement(stmt);
        }

        Ok(graph)
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
                                    &*self.operators[out].succs.len().to_string(),
                                    curr_arrow.arrow.span(),
                                )
                            });
                            let dst_port = curr_arrow.dst.map(|x| x.index).unwrap_or_else(|| {
                                LitInt::new(
                                    &*self.operators[inn].preds.len().to_string(),
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

                                if let Some((old_a, _)) =
                                    self.operators[out].succs.remove_entry(&src_a)
                                {
                                    emit_conflict("Output", &old_a, &src_a);
                                }
                                self.operators[out].succs.insert(src_a, (inn, dst_a));

                                if let Some((old_b, _)) =
                                    self.operators[inn].preds.remove_entry(&dst_b)
                                {
                                    emit_conflict("Input", &old_b, &dst_b);
                                }
                                self.operators[inn].preds.insert(dst_b, (out, src_b));
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
                let (preds, succs) = Default::default();
                let op_info = OpInfo {
                    operator,
                    preds,
                    succs,
                };
                let port = self.operators.insert(op_info);
                Ports {
                    inn: Some(port),
                    out: Some(port),
                }
            }
        }
    }

    pub fn mermaid_string(&self) -> String {
        let mut string = String::new();
        self.write_mermaid(&mut string).unwrap();
        string
    }

    pub fn write_mermaid(&self, write: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(write, "flowchart TB")?;
        for (key, op) in self.operators.iter() {
            writeln!(
                write,
                r#"    {}["{}"]"#,
                key.data().as_ffi(),
                op.operator
                    .to_token_stream()
                    .to_string()
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;")
            )?;
        }
        writeln!(write)?;
        for (src_key, op) in self.operators.iter() {
            for (_src_port, (dst_key, _dst_port)) in op.succs.iter() {
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
}

#[derive(Clone, Copy, Debug)]
struct Ports {
    inn: Option<DefaultKey>,
    out: Option<DefaultKey>,
}

struct OpInfo {
    operator: Operator,
    preds: HashMap<LitInt, (DefaultKey, LitInt)>,
    succs: HashMap<LitInt, (DefaultKey, LitInt)>,
}
impl std::fmt::Debug for OpInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpInfo")
            .field("operator (span)", &self.operator.span())
            .field("preds", &self.preds)
            .field("succs", &self.succs)
            .finish()
    }
}

/// Helper struct which displays the span as `path:row:col` for human reading/IDE linking.
/// Example: `hydroflow\tests\surface_syntax.rs:42:18`.
struct PrettySpan(Span);
impl std::fmt::Display for PrettySpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let span = self.0.unwrap();
        write!(
            f,
            "{}:{}:{}",
            span.source_file().path().display(),
            span.start().line,
            span.start().column
        )
    }
}
