use std::collections::{HashMap, HashSet};

use hydroflow_lang::{
    graph::flat_graph::FlatGraph,
    parse::{
        ArrowConnector, IndexInt, Indexing, NamedHfStatement, Operator, Pipeline, PipelineLink,
    },
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, punctuated::Punctuated, token::Paren, Token};

mod grammar;

use grammar::datalog::*;

fn gen_datalog_program(literal: proc_macro2::Literal, root: TokenStream) -> syn::Stmt {
    let str_node: syn::LitStr = parse_quote!(#literal);
    let actual_str = str_node.value();
    let program: Program = grammar::datalog::parse(&actual_str).unwrap();

    let inputs = program
        .rules
        .iter()
        .filter_map(|decl| match decl {
            Declaration::Input(_, ident) => Some(ident.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    let outputs = program
        .rules
        .iter()
        .filter_map(|decl| match decl {
            Declaration::Output(_, ident) => Some(ident.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    let rules = program
        .rules
        .iter()
        .filter_map(|decl| match decl {
            Declaration::Rule(rule) => Some(rule.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    let mut flat_graph = FlatGraph::default();
    let mut tee_counter = HashMap::new();

    let mut created_rules = HashSet::new();
    for decl in program.rules {
        let target_ident = match decl {
            Declaration::Input(_, ident) => ident.clone(),
            Declaration::Output(_, ident) => ident.clone(),
            Declaration::Rule(rule) => rule.target.name.clone(),
        };

        if !created_rules.contains(&target_ident) {
            created_rules.insert(target_ident.clone());
            flat_graph.add_statement(hydroflow_lang::parse::HfStatement::Named(
                NamedHfStatement {
                    name: syn::Ident::new(&target_ident.name, Span::call_site()),
                    equals: Token![=](Span::call_site()),
                    pipeline: Pipeline::Operator(Operator {
                        path: parse_quote!(tee),
                        paren_token: Paren::default(),
                        args: Punctuated::new(),
                    }),
                },
            ));
        }
    }

    for target in inputs {
        let target_ident = syn::Ident::new(&format!("{}_recv", &target.name), Span::call_site());
        flat_graph.add_statement(hydroflow_lang::parse::HfStatement::Pipeline(
            Pipeline::Link(PipelineLink {
                lhs: Box::new(Pipeline::Operator(Operator {
                    path: parse_quote!(recv_stream),
                    paren_token: Paren::default(),
                    args: vec![parse_quote!(#target_ident)]
                        .iter()
                        .cloned::<syn::Expr>()
                        .collect(),
                })),
                connector: ArrowConnector {
                    src: None,
                    arrow: Token![->](Span::call_site()),
                    dst: None,
                },
                rhs: Box::new(Pipeline::Name(syn::Ident::new(
                    &target.name,
                    Span::call_site(),
                ))),
            }),
        ));
    }

    for target in outputs {
        let tee_index = tee_counter.entry(target.name.clone()).or_insert(0);
        let my_tee_index = *tee_index;
        *tee_index += 1;

        flat_graph.add_statement(hydroflow_lang::parse::HfStatement::Pipeline(
            Pipeline::Link(PipelineLink {
                lhs: Box::new(Pipeline::Name(syn::Ident::new(
                    &target.name,
                    Span::call_site(),
                ))),
                connector: ArrowConnector {
                    src: Some(Indexing {
                        bracket_token: syn::token::Bracket::default(),
                        index: IndexInt {
                            value: my_tee_index,
                            span: Span::call_site(),
                        },
                    }),
                    arrow: Token![->](Span::call_site()),
                    dst: None,
                },
                rhs: Box::new(Pipeline::Operator(Operator {
                    path: parse_quote!(for_each),
                    paren_token: Paren::default(),
                    args: vec![parse_quote!(|v| println!("{:?}", v))]
                        .iter()
                        .cloned::<syn::Expr>()
                        .collect(),
                })),
            }),
        ));
    }

    for rule in rules {
        let target = rule.target.name;
        let target_ident = syn::Ident::new(&target.name, Span::call_site());
        let sources: Vec<Target> = rule.sources.to_vec();

        // TODO(shadaj): more than two sources, nested join
        let mut identifier_to_bindings = HashMap::new();
        for (source_idx, source) in sources.iter().enumerate() {
            for (i, param) in source.fields.iter().enumerate() {
                let entry = identifier_to_bindings
                    .entry(param.clone())
                    .or_insert_with(HashMap::new);
                entry.insert(source_idx, i);
            }
        }

        let mut sorted_identifiers = identifier_to_bindings.keys().cloned().collect::<Vec<_>>();
        sorted_identifiers.sort_by_key(|ident| ident.name.clone());

        let identifiers_to_join = sorted_identifiers
            .iter()
            .filter_map(|ident| {
                let bindings = identifier_to_bindings.get(ident).unwrap();
                if bindings.len() > 1 {
                    Some((ident.clone(), bindings.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let join_node = syn::Ident::new(&format!("{}_join", target_ident), Span::call_site());

        let output_data = rule
            .target
            .fields
            .iter()
            .map(|field| {
                let bindings = identifier_to_bindings.get(field).unwrap();
                let source = bindings.keys().min().unwrap();
                let source_expr: syn::Expr = if sources.len() == 1 {
                    parse_quote!(v)
                } else {
                    let pair_idx = syn::Index::from(*source);
                    parse_quote!(kv.1.#pair_idx)
                };

                let source_col_idx = syn::Index::from(*bindings.get(source).unwrap());

                parse_quote!(#source_expr.#source_col_idx)
            })
            .collect::<Vec<syn::Expr>>();

        // TODO(shadaj): this is all a hack, will fall apart when we have more than usize types
        let source_types = sources
            .iter()
            .map(|source| {
                let col_types = source
                    .fields
                    .iter()
                    .map(|_| parse_quote!(usize))
                    .collect::<Vec<syn::Type>>();

                parse_quote!((#(#col_types),*))
            })
            .collect::<Vec<syn::Type>>();

        let key_type = identifiers_to_join
            .iter()
            .map(|(_, _)| parse_quote!(usize))
            .collect::<Vec<syn::Type>>();

        let after_join_map: syn::Expr = if sources.len() == 1 {
            parse_quote!(|v| (#(#output_data),*))
        } else {
            parse_quote!(|kv: ((#(#key_type),*), (#(#source_types),*))| (#(#output_data),*))
        };

        let after_join = Pipeline::Link(PipelineLink {
            lhs: Box::new(Pipeline::Operator(Operator {
                path: parse_quote!(map),
                paren_token: Paren::default(),
                args: vec![after_join_map].iter().cloned::<syn::Expr>().collect(),
            })),
            connector: ArrowConnector {
                src: None,
                arrow: Token![->](Span::call_site()),
                dst: None,
            },
            rhs: Box::new(Pipeline::Name(target_ident.clone())),
        });

        let join_and_map = if sources.len() == 1 {
            after_join
        } else {
            Pipeline::Link(PipelineLink {
                lhs: Box::new(Pipeline::Operator(Operator {
                    path: parse_quote!(join),
                    paren_token: Paren::default(),
                    args: Punctuated::new(),
                })),
                connector: ArrowConnector {
                    src: None,
                    arrow: Token![->](Span::call_site()),
                    dst: None,
                },
                rhs: Box::new(after_join),
            })
        };

        flat_graph.add_statement(hydroflow_lang::parse::HfStatement::Named(
            NamedHfStatement {
                name: join_node.clone(),
                equals: Token![=](Span::call_site()),
                pipeline: join_and_map,
            },
        ));

        for (source_i, source) in sources.iter().enumerate() {
            let hash_keys: Vec<syn::Expr> = identifiers_to_join
                .iter()
                .map(|(ident, bindings)| {
                    if let Some(idx) = bindings.get(&source_i) {
                        let idx_ident = syn::Index::from(*idx);
                        parse_quote!(v.#idx_ident)
                    } else {
                        panic!("Could not find key that is being joined on: {:?}", ident);
                    }
                })
                .collect::<Vec<_>>();

            let tee_index = tee_counter.entry(source.name.name.clone()).or_insert(0);
            let my_tee_index = *tee_index;
            *tee_index += 1;

            let source_data_types = source
                .fields
                .iter()
                .map(|_| parse_quote!(usize))
                .collect::<Vec<syn::Type>>();

            flat_graph.add_statement(hydroflow_lang::parse::HfStatement::Pipeline(
                Pipeline::Link(PipelineLink {
                    lhs: Box::new(Pipeline::Name(syn::Ident::new(
                        &source.name.name,
                        Span::call_site(),
                    ))),
                    connector: ArrowConnector {
                        src: Some(Indexing {
                            bracket_token: syn::token::Bracket::default(),
                            index: IndexInt {
                                value: my_tee_index,
                                span: Span::call_site(),
                            }
                        }),
                        arrow: Token![->](Span::call_site()),
                        dst: None,
                    },
                    rhs: Box::new(if sources.len() == 1 {
                        Pipeline::Name(join_node.clone())
                    } else {
                        Pipeline::Link(PipelineLink {
                            lhs: Box::new(Pipeline::Operator(Operator {
                                path: parse_quote!(map),
                                paren_token: Paren::default(),
                                args: vec![parse_quote!(|v: (#(#source_data_types),*)| ((#(#hash_keys),*), v))]
                                    .iter()
                                    .cloned::<syn::Expr>()
                                    .collect(),
                            })),
                            connector: ArrowConnector {
                                src: None,
                                arrow: Token![->](Span::call_site()),
                                dst: Some(Indexing {
                                    bracket_token: syn::token::Bracket::default(),
                                    index: IndexInt {
                                        value: source_i,
                                        span: Span::call_site(),
                                    },
                                }),
                            },
                            rhs: Box::new(Pipeline::Name(join_node.clone())),
                        })
                    }),
                }),
            ));
        }
    }

    println!("{}", flat_graph.surface_syntax_string());

    let code_tokens = flat_graph
        .into_partitioned_graph()
        .expect("failed to partition")
        .as_code(root);

    syn::parse_quote!({
        #code_tokens
    })
}

#[proc_macro]
pub fn datalog(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = proc_macro2::TokenStream::from(item);
    let literal: proc_macro2::Literal = syn::parse_quote! {
        #item
    };

    let hydroflow_crate = proc_macro_crate::crate_name("hydroflow")
        .expect("hydroflow should be present in `Cargo.toml`");
    let root = match hydroflow_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { hydroflow },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote! { #ident }
        }
    };

    proc_macro::TokenStream::from(gen_datalog_program(literal, root).to_token_stream())
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{Read, Write};
    use std::process::Command;

    use quote::ToTokens;
    use syn::parse_quote;
    use tempfile::tempdir;

    use super::gen_datalog_program;

    fn rustfmt_code(code: &str) -> String {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("temp.rs");
        let mut file = File::create(file_path.clone()).unwrap();

        writeln!(file, "{}", code).unwrap();
        drop(file);

        Command::new("rustfmt")
            .arg(file_path.to_str().unwrap())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        let mut file = File::open(file_path).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        drop(file);
        dir.close().unwrap();
        data
    }

    #[test]
    fn minimal_program() {
        let out = &gen_datalog_program(
            parse_quote!(
                r#"
                .input in
                .output out

                out(y, x) :- in(x, y).
                "#
            ),
            quote::quote! { hydroflow },
        );

        let wrapped: syn::Item = parse_quote! {
            fn main() {
                #out
            }
        };

        insta::assert_display_snapshot!(rustfmt_code(&wrapped.to_token_stream().to_string()));
    }

    // non-deterministic codegen
    // #[test]
    // fn join_with_self() {
    //     let out = &gen_datalog_program(
    //         parse_quote!(
    //             r#"
    //             .input in
    //             .output out

    //             out(x, y) :- in(x, y), in(y, x).
    //             "#
    //         ),
    //         quote::quote! { hydroflow },
    //     );

    //     let wrapped: syn::Item = parse_quote! {
    //         fn main() {
    //             #out
    //         }
    //     };

    //     insta::assert_display_snapshot!(rustfmt_code(&wrapped.to_token_stream().to_string()));
    // }

    #[test]
    fn join_with_other() {
        let out = &gen_datalog_program(
            parse_quote!(
                r#"
                .input in1
                .input in2
                .output out

                out(x, y) :- in1(x, y), in2(y, x).
                "#
            ),
            quote::quote! { hydroflow },
        );

        let wrapped: syn::Item = parse_quote! {
            fn main() {
                #out
            }
        };

        insta::assert_display_snapshot!(rustfmt_code(&wrapped.to_token_stream().to_string()));
    }
}
