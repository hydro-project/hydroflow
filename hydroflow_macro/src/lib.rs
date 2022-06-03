use std::collections::HashMap;

use proc_macro2::Literal;
use quote::{quote, ToTokens};
use slotmap::{DefaultKey, SlotMap};
use syn::{parse_macro_input, spanned::Spanned, Ident};

mod parse;
use parse::{HfCode, Operator, Pipeline};

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

    pub fn add_pipeline(&mut self, pipeline: Pipeline) -> PipelinePtrPair {
        match pipeline {
            Pipeline::Chain(chain_pipeline) => {
                let mut pipe_ptr = PipelinePtrPair::default();

                if chain_pipeline.leading_arrow.is_some() {
                    // TODO(mingwei). this should do something
                }
                for elem in chain_pipeline.elems {
                    let sub_ptr = self.add_pipeline(elem);

                    if let (Some(out), Some(inn)) = (pipe_ptr.out, sub_ptr.inn) {
                        self.operators[out].succs.push(inn);
                        self.operators[inn].preds.push(out);
                    }

                    pipe_ptr.inn = pipe_ptr.inn.or(sub_ptr.inn);
                    pipe_ptr.out = sub_ptr.out;
                }

                pipe_ptr
            }
            Pipeline::Multiple(multiple) => {
                // TODO: match on name.
                let (preds, succs) = Default::default();
                let key = self.operators.insert(OpInfo {
                    operator: None,
                    preds,
                    succs,
                });
                PipelinePtrPair {
                    inn: Some(key),
                    out: Some(key),
                }
            }
            Pipeline::Ident(ident) => {
                *self
                    .names
                    .get(&ident)
                    .unwrap_or_else(|| panic!("Failed to find name: {}", ident))
                // TODO(mingwei): error reporting
            }
            Pipeline::Operator(operator) => {
                let (preds, succs) = Default::default();
                let key = self.operators.insert(OpInfo {
                    operator: Some(operator),
                    preds,
                    succs,
                });
                PipelinePtrPair {
                    inn: Some(key),
                    out: Some(key),
                }
            }
        }
    }
}

struct OpInfo {
    operator: Option<Operator>, // TODO(handle n-ary as operators)
    preds: Vec<DefaultKey>,
    succs: Vec<DefaultKey>,
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
        let pipe_ptr = graph.add_pipeline(stmt.pipeline);

        match (stmt.name, stmt.equals) {
            (Some(name), Some(_eq)) => {
                graph.names.insert(name, pipe_ptr);
            }
            (None, None) => {
                // Anonymous stmt.
            }
            _ => {
                panic!("name and equal must be together!") // TODO(mingwei).
            }
        }
    }

    graph
}

fn graph_to_hfcode(input: Graph) -> HfCode {
    todo!();
    HfCode {
        statements: Default::default(),
    }
}
