use std::collections::{BTreeMap, HashMap, HashSet};

use hydroflow_lang::{graph::flat_graph::FlatGraph, parse::Pipeline};
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

    for rule in rules {
        generate_join(rule, &mut flat_graph, &mut tee_counter, &mut merge_counter);
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

fn generate_join(
    rule: &Rule,
    flat_graph: &mut FlatGraph,
    tee_counter: &mut HashMap<String, i32>,
    merge_counter: &mut HashMap<String, i32>,
) {
    let target = &rule.target.name;
    let target_ident = syn::Ident::new(&target.name, Span::call_site());

    let sources: Vec<Target> = rule.sources.to_vec();

    // TODO(shadaj): more than two sources, nested join
    let mut identifier_to_bindings = BTreeMap::new();
    for (source_idx, source) in sources.iter().enumerate() {
        for (i, param) in source.fields.iter().enumerate() {
            let entry = identifier_to_bindings
                .entry(param.clone())
                .or_insert_with(BTreeMap::new);
            entry.insert(source_idx, i);
        }
    }

    let identifiers_to_join = identifier_to_bindings
        .keys()
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

            parse_quote!((#(#col_types, )*))
        })
        .collect::<Vec<syn::Type>>();

    let key_type = identifiers_to_join
        .iter()
        .map(|(_, _)| parse_quote!(usize))
        .collect::<Vec<syn::Type>>();

    let after_join_map: syn::Expr = if sources.len() == 1 {
        parse_quote!(|v| (#(#output_data, )*))
    } else {
        parse_quote!(|kv: ((#(#key_type, )*), (#(#source_types, )*))| (#(#output_data, )*))
    };

    let merge_index = merge_counter.entry(target.name.clone()).or_insert(0);
    let my_merge_index = *merge_index;
    *merge_index += 1;

    let my_merge_index_lit = syn::LitInt::new(&format!("{}", my_merge_index), Span::call_site());

    let after_join: Pipeline = parse_quote! {
        map(#after_join_map) -> [#my_merge_index_lit] #target_ident
    };

    let join_and_map = if sources.len() == 1 {
        after_join
    } else {
        parse_quote!(join() -> #after_join)
    };

    flat_graph.add_statement(parse_quote!(#join_node = #join_and_map));

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
            .collect();

        let tee_index = tee_counter.entry(source.name.name.clone()).or_insert(0);
        let my_tee_index = *tee_index;
        *tee_index += 1;

        let my_tee_index_lit = syn::LitInt::new(&format!("{}", my_tee_index), Span::call_site());

        let source_data_types = source
            .fields
            .iter()
            .map(|_| parse_quote!(usize))
            .collect::<Vec<syn::Type>>();

        let source_ident = syn::Ident::new(&source.name.name, Span::call_site());

        let transform_join_source = if sources.len() == 1 {
            Pipeline::Name(join_node.clone())
        } else {
            let source_i_lit = syn::LitInt::new(&format!("{}", source_i), Span::call_site());
            parse_quote! {
                map(|v: (#(#source_data_types, )*)| ((#(#hash_keys, )*), v)) -> [#source_i_lit] #join_node
            }
        };

        flat_graph.add_statement(parse_quote! {
            #source_ident [#my_tee_index_lit] -> #transform_join_source
        });
    }
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
