#![feature(proc_macro_diagnostic)]

use std::collections::HashMap;

use proc_macro2::{Literal, Span};
use quote::{quote, ToTokens};
use slotmap::{DefaultKey, SlotMap};
use syn::{parse_macro_input, spanned::Spanned, Ident, LitInt};

mod parse;
use parse::{HfCode, HfStatement, NamePipeline, NamedHfStatement, Operator, Pipeline};

#[proc_macro]
pub fn hydroflow_parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as HfCode);
    // // input.into_token_stream().into()

    let graph = hfcode_to_graph(input);

    // let debug = format!("{:#?}", graph);
    let mut debug = String::new();
    graph.write_graph(&mut debug).unwrap();

    let lit = Literal::string(&*debug);

    quote! { println!("{}", #lit); }.into()
}

#[derive(Clone, Copy, Debug, Default)]
struct PipelinePtrPair {
    inn: Option<DefaultKey>,
    out: Option<DefaultKey>,
}

#[derive(Clone, Debug, Default)]
struct PipelinePtrPairIndexed {
    pair: PipelinePtrPair,
    inn_idx: Option<LitInt>,
    out_idx: Option<LitInt>,
}

#[derive(Debug, Default)]
struct Graph {
    operators: SlotMap<DefaultKey, OpInfo>,
    names: HashMap<Ident, PipelinePtrPair>,
}
impl Graph {
    pub fn write_graph(&self, mut write: impl std::fmt::Write) -> std::fmt::Result {
        for (key, op) in self.operators.iter() {
            writeln!(
                write,
                "{:?}: {}",
                key,
                op.operator.to_token_stream().to_string()
            )?;
            writeln!(write, "    preds: {:?}", op.preds)?;
            writeln!(write, "    succs: {:?}", op.succs)?;
            writeln!(write)?;
        }
        Ok(())
    }

    fn add_statement(&mut self, stmt: HfStatement) {
        match stmt {
            parse::HfStatement::Named(NamedHfStatement {
                name,
                equals: _,
                pipeline,
            }) => {
                if let Some(pipe_ptr) = self.add_pipeline(pipeline) {
                    self.names.insert(name, pipe_ptr.pair);
                }
            }
            parse::HfStatement::Pipeline(pipeline) => {
                self.add_pipeline(pipeline);
            }
        }
    }

    pub fn add_pipeline(&mut self, pipeline: Pipeline) -> Option<PipelinePtrPairIndexed> {
        match pipeline {
            Pipeline::Chain(chain_pipeline) => {
                let mut pipe_ptr: Option<PipelinePtrPairIndexed> = None;

                if chain_pipeline.leading_arrow.is_some() {
                    // TODO(mingwei). this should do something
                }
                for elem in chain_pipeline.elems {
                    let sub_ptr = self.add_pipeline(elem);
                    if let Some(sub_ptr) = sub_ptr {
                        let pipe_ptr = pipe_ptr.get_or_insert_with(Default::default);

                        if let (Some(out), Some(inn)) = (pipe_ptr.pair.out, sub_ptr.pair.inn) {
                            let succs = &mut self.operators[out].succs;
                            if let Some(old_inn) = succs.insert(
                                sub_ptr.inn_idx.unwrap_or(LitInt::new(
                                    &*succs.len().to_string(),
                                    Span::call_site(/* TODO(mingwei): actual span info */),
                                )),
                                inn,
                            ) {
                                panic!("TODO(mingwei): error message on span A {:?}", old_inn);
                            }

                            let preds = &mut self.operators[inn].preds;
                            if let Some(old_out) = preds.insert(
                                pipe_ptr.out_idx.clone().unwrap_or(LitInt::new(
                                    &*preds.len().to_string(),
                                    Span::call_site(/* TODO(mingwei): actual span info */),
                                )),
                                out,
                            ) {
                                panic!("TODO(mingwei): error message on span B {:?}", old_out);
                            }
                        }

                        pipe_ptr.pair.inn = pipe_ptr.pair.inn.or(sub_ptr.pair.inn);
                        pipe_ptr.pair.out = sub_ptr.pair.out;
                    }
                }

                pipe_ptr
                // pipe_ptr.map(|pair| PipelinePtrPairIndexed {
                //     pair,
                //     inn_idx: None,
                //     out_idx: None,
                // })
            }
            // Pipeline::Multiple(multiple) => {
            //     // TODO: match on name.
            //     let (preds, succs) = Default::default();
            //     let key = self.operators.insert(OpInfo {
            //         operator: None,
            //         preds,
            //         succs,
            //     });
            //     PipelinePtrPair {
            //         inn: Some(key),
            //         out: Some(key),
            //     }
            // }
            // Pipeline::Ident(ident) => {
            //     *self
            //         .names
            //         .get(&ident)
            //         .unwrap_or_else(|| panic!("Failed to find name: {}", ident))
            //     // TODO(mingwei): error reporting
            // }
            Pipeline::Name(NamePipeline {
                prefix,
                name,
                suffix,
            }) => {
                // TODO(mingwei): PREFIX AND SUFFIX
                let opt_pair = self.names.get(&name).copied();
                if let Some(pair) = opt_pair {
                    Some(PipelinePtrPairIndexed {
                        pair,
                        inn_idx: prefix.map(|x| x.index),
                        out_idx: suffix.map(|x| x.index),
                    })
                } else {
                    name.span()
                        .unwrap()
                        .error(format!("Cannot find name `{}`.", name))
                        .emit();
                    None
                }
            }
            Pipeline::Operator(operator) => {
                let (preds, succs) = Default::default();
                let key = self.operators.insert(OpInfo {
                    operator: Some(operator),
                    preds,
                    succs,
                });
                Some(PipelinePtrPairIndexed {
                    pair: PipelinePtrPair {
                        inn: Some(key),
                        out: Some(key),
                    },
                    inn_idx: None,
                    out_idx: None,
                })
            }
        }
    }
}

struct OpInfo {
    operator: Option<Operator>, // TODO(handle n-ary as operators)
    preds: HashMap<LitInt, DefaultKey>,
    succs: HashMap<LitInt, DefaultKey>,
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

fn hfcode_to_graph(input: HfCode) -> Graph {
    let mut graph = Graph::default();

    for stmt in input.statements {
        graph.add_statement(stmt);

        // let pipe_ptr = graph.add_pipeline(stmt.pipeline);

        // match (stmt.name, stmt.equals) {
        //     (Some(name), Some(_eq)) => {
        //         graph.names.insert(name, pipe_ptr);
        //     }
        //     (None, None) => {
        //         // Anonymous stmt.
        //     }
        //     _ => {
        //         panic!("name and equal must be together!") // TODO(mingwei).
        //     }
        // }
    }

    graph
}

fn graph_to_hfcode(input: Graph) -> HfCode {
    todo!();
    HfCode {
        statements: Default::default(),
    }
}
