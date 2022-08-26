use std::collections::{BTreeMap, HashMap, HashSet};

use hydroflow_lang::{
    graph::flat_graph::FlatGraph,
    parse::{ArrowConnector, IndexInt, Indexing, Pipeline, PipelineLink},
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse_quote;

mod grammar;

use grammar::datalog::*;

fn gen_datalog_program(literal: proc_macro2::Literal, root: TokenStream) -> syn::Stmt {
    let str_node: syn::LitStr = parse_quote!(#literal);
    let actual_str = str_node.value();
    let program: Program = grammar::datalog::parse(&actual_str).unwrap();

    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    let mut rules = Vec::new();

    for stmt in &program.rules {
        match stmt {
            Declaration::Input(_, ident) => inputs.push(ident),
            Declaration::Output(_, ident) => outputs.push(ident),
            Declaration::Rule(rule) => rules.push(rule),
        }
    }

    let mut flat_graph = FlatGraph::default();
    let mut tee_counter = HashMap::new();
    let mut merge_counter = HashMap::new();

    let mut created_rules = HashSet::new();
    for decl in &program.rules {
        let target_ident = match decl {
            Declaration::Input(_, ident) => ident.clone(),
            Declaration::Output(_, ident) => ident.clone(),
            Declaration::Rule(rule) => rule.target.name.clone(),
        };

        if !created_rules.contains(&target_ident) {
            created_rules.insert(target_ident.clone());
            let name = syn::Ident::new(&target_ident.name, Span::call_site());
            flat_graph.add_statement(parse_quote!(#name = merge() -> tee()));
        }
    }

    for target in inputs {
        let target_ident = syn::Ident::new(&target.name, Span::call_site());

        let merge_index = merge_counter.entry(target.name.clone()).or_insert(0);
        let my_merge_index = *merge_index;
        *merge_index += 1;

        let my_merge_index_lit =
            syn::LitInt::new(&format!("{}", my_merge_index), Span::call_site());
        let name = syn::Ident::new(&target.name, Span::call_site());

        flat_graph.add_statement(parse_quote! {
            recv_stream(#target_ident) -> [#my_merge_index_lit] #name
        });
    }

    for target in outputs {
        let tee_index = tee_counter.entry(target.name.clone()).or_insert(0);
        let my_tee_index = *tee_index;
        *tee_index += 1;

        let out_send_ident = syn::Ident::new(&target.name, Span::call_site());

        let my_tee_index_lit = syn::LitInt::new(&format!("{}", my_tee_index), Span::call_site());
        let target_ident = syn::Ident::new(&target.name, Span::call_site());

        flat_graph.add_statement(parse_quote! {
            #target_ident [#my_tee_index_lit] -> for_each(|v| #out_send_ident.send(v).unwrap())
        });
    }

    let mut next_join_idx = 0;
    for rule in rules {
        generate_join(
            rule,
            &mut flat_graph,
            &mut tee_counter,
            &mut merge_counter,
            &mut next_join_idx,
        );
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

enum JoinPlan {
    Source(usize),
    Join(Box<JoinPlan>, Box<JoinPlan>),
}

// outputs the identifier for the join node and a mapping from rule identifiers to indices in the join output tuple
fn expand_join_plan(
    plan: &JoinPlan,
    all_sources: &[Target],
    flat_graph: &mut FlatGraph,
    tee_counter: &mut HashMap<String, usize>,
    merge_counter: &mut HashMap<String, usize>,
    next_join_idx: &mut usize,
) -> ((syn::Ident, Option<usize>), BTreeMap<syn::Ident, usize>) {
    match plan {
        JoinPlan::Source(idx) => {
            let target = &all_sources[*idx];
            let mut mapping = BTreeMap::new();
            for (i, ident) in target.fields.iter().enumerate() {
                mapping.insert(syn::Ident::new(&ident.name, Span::call_site()), i);
                // TODO(shadaj): if there is already an entry in mapping that means filter
            }

            let tee_index = tee_counter.entry(target.name.name.clone()).or_insert(0);
            let my_tee_index = *tee_index;
            *tee_index += 1;

            (
                (
                    syn::Ident::new(&target.name.name, Span::call_site()),
                    Some(my_tee_index),
                ),
                mapping,
            )
        }
        JoinPlan::Join(lhs, rhs) => {
            let ((left_node, left_node_tee_idx), left_idents) = expand_join_plan(
                lhs,
                all_sources,
                flat_graph,
                tee_counter,
                merge_counter,
                next_join_idx,
            );
            let ((right_node, right_node_tee_idx), right_idents) = expand_join_plan(
                rhs,
                all_sources,
                flat_graph,
                tee_counter,
                merge_counter,
                next_join_idx,
            );

            let my_idx = *next_join_idx;
            *next_join_idx += 1;

            let left_identifiers = left_idents.keys().collect::<HashSet<_>>();
            let identifiers_to_join = right_idents
                .keys()
                .filter(|i| left_identifiers.contains(i))
                .collect::<Vec<_>>();

            let mut output_data: Vec<syn::Expr> = vec![];
            let mut ident_to_index = BTreeMap::new();

            for (ident, source_idx) in left_idents
                .keys()
                .map(|l| (l, syn::Index::from(0)))
                .chain(right_idents.keys().map(|l| (l, syn::Index::from(1))))
            {
                if !ident_to_index.contains_key(ident) {
                    let source_expr: syn::Expr = parse_quote!(kv.1.#source_idx);
                    let bindings = if source_idx.index == 0 {
                        &left_idents
                    } else {
                        &right_idents
                    };

                    let source_col_idx = syn::Index::from(*bindings.get(ident).unwrap());

                    ident_to_index.insert(ident.clone(), output_data.len());
                    output_data.push(parse_quote!(#source_expr.#source_col_idx));
                }
            }

            // TODO(shadaj): dedup
            let left_types = left_idents
                .iter()
                .map(|_| parse_quote!(_))
                .collect::<Vec<syn::Type>>();
            let left_tuple: syn::Type = parse_quote!((#(#left_types, )*));

            let right_types = right_idents
                .iter()
                .map(|_| parse_quote!(_))
                .collect::<Vec<syn::Type>>();
            let right_tuple: syn::Type = parse_quote!((#(#right_types, )*));

            let key_type = identifiers_to_join
                .iter()
                .map(|_| parse_quote!(_))
                .collect::<Vec<syn::Type>>();

            let after_join_map: syn::Expr = parse_quote!(|kv: ((#(#key_type, )*), (#left_tuple, #right_tuple))| (#(#output_data, )*));

            let join_node = syn::Ident::new(&format!("join_{}", my_idx), Span::call_site());
            flat_graph.add_statement(parse_quote!(#join_node = join() -> map(#after_join_map)));

            // TODO(shadaj): dedup
            let hash_keys_left: Vec<syn::Expr> = identifiers_to_join
                .iter()
                .map(|ident| {
                    if let Some(idx) = left_idents.get(ident) {
                        let idx_ident = syn::Index::from(*idx);
                        parse_quote!(v.#idx_ident)
                    } else {
                        panic!("Could not find key that is being joined on: {:?}", ident);
                    }
                })
                .collect();

            flat_graph.add_statement(hydroflow_lang::parse::HfStatement::Pipeline(
                Pipeline::Link(PipelineLink {
                    lhs: Box::new(parse_quote!(#left_node)),
                    connector: ArrowConnector {
                        src: left_node_tee_idx.map(|i| Indexing {
                            bracket_token: syn::token::Bracket::default(),
                            index: IndexInt {
                                value: i,
                                span: Span::call_site(),
                            },
                        }),
                        arrow: parse_quote!(->),
                        dst: None,
                    },
                    rhs: Box::new(parse_quote! {
                        map(|v: #left_tuple| ((#(#hash_keys_left, )*), v)) -> [0] #join_node
                    }),
                }),
            ));

            let hash_keys_right: Vec<syn::Expr> = identifiers_to_join
                .iter()
                .map(|ident| {
                    if let Some(idx) = right_idents.get(ident) {
                        let idx_ident = syn::Index::from(*idx);
                        parse_quote!(v.#idx_ident)
                    } else {
                        panic!("Could not find key that is being joined on: {:?}", ident);
                    }
                })
                .collect();

            flat_graph.add_statement(hydroflow_lang::parse::HfStatement::Pipeline(
                Pipeline::Link(PipelineLink {
                    lhs: Box::new(parse_quote!(#right_node)),
                    connector: ArrowConnector {
                        src: right_node_tee_idx.map(|i| Indexing {
                            bracket_token: syn::token::Bracket::default(),
                            index: IndexInt {
                                value: i,
                                span: Span::call_site(),
                            },
                        }),
                        arrow: parse_quote!(->),
                        dst: None,
                    },
                    rhs: Box::new(parse_quote! {
                        map(|v: #right_tuple| ((#(#hash_keys_right, )*), v)) -> [1] #join_node
                    }),
                }),
            ));

            ((join_node, None), ident_to_index)
        }
    }
}

fn generate_join(
    rule: &Rule,
    flat_graph: &mut FlatGraph,
    tee_counter: &mut HashMap<String, usize>,
    merge_counter: &mut HashMap<String, usize>,
    next_join_idx: &mut usize,
) {
    let target = &rule.target.name;
    let target_ident = syn::Ident::new(&target.name, Span::call_site());

    let sources: Vec<Target> = rule.sources.to_vec();

    // TODO(shadaj): smarter plans
    let plan = sources
        .iter()
        .enumerate()
        .map(|(i, _)| JoinPlan::Source(i))
        .reduce(|a, b| JoinPlan::Join(Box::new(a), Box::new(b)))
        .unwrap();

    let ((join_node, join_tee), ident_mapping) = expand_join_plan(
        &plan,
        &sources,
        flat_graph,
        tee_counter,
        merge_counter,
        next_join_idx,
    );

    let output_data = rule
        .target
        .fields
        .iter()
        .map(|field| {
            let col = ident_mapping
                .get(&syn::Ident::new(&field.name, Span::call_site()))
                .unwrap();
            let source_col_idx = syn::Index::from(*col);

            parse_quote!(row.#source_col_idx)
        })
        .collect::<Vec<syn::Expr>>();

    let row_type = ident_mapping
        .iter()
        .map(|_| parse_quote!(_))
        .collect::<Vec<syn::Type>>();

    let after_join_map: syn::Expr = parse_quote!(|row: (#(#row_type, )*)| (#(#output_data, )*));

    let merge_index = merge_counter.entry(target.name.clone()).or_insert(0);
    let my_merge_index = *merge_index;
    *merge_index += 1;

    let my_merge_index_lit = syn::LitInt::new(&format!("{}", my_merge_index), Span::call_site());

    let after_join: Pipeline = parse_quote! {
        map(#after_join_map) -> [#my_merge_index_lit] #target_ident
    };

    flat_graph.add_statement(hydroflow_lang::parse::HfStatement::Pipeline(
        Pipeline::Link(PipelineLink {
            lhs: Box::new(parse_quote!(#join_node)),
            connector: ArrowConnector {
                src: join_tee.map(|i| Indexing {
                    bracket_token: syn::token::Bracket::default(),
                    index: IndexInt {
                        value: i,
                        span: Span::call_site(),
                    },
                }),
                arrow: parse_quote!(->),
                dst: None,
            },
            rhs: Box::new(after_join),
        }),
    ));
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
                .input input
                .output out

                out(y, x) :- input(x, y).
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

    #[test]
    fn join_with_self() {
        let out = &gen_datalog_program(
            parse_quote!(
                r#"
                .input input
                .output out

                out(x, y) :- input(x, y), input(y, x).
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

    #[test]
    fn multiple_contributors() {
        let out = &gen_datalog_program(
            parse_quote!(
                r#"
                .input in1
                .input in2
                .output out

                out(x, y) :- in1(x, y).
                out(x, y) :- in2(y, x).
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

    #[test]
    fn single_column_program() {
        let out = &gen_datalog_program(
            parse_quote!(
                r#"
                .input in1
                .input in2
                .output out

                out(x) :- in1(x), in2(x).
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
