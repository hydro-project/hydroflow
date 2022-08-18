use hydroflow_lang::{
    graph::flat_graph::FlatGraph,
    parse::{ArrowConnector, NamedHfStatement, Operator, Pipeline, PipelineLink},
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, token::Paren, Token};

#[rust_sitter::grammar("datalog")]
#[allow(dead_code)]
mod datalog_grammar {
    #[rust_sitter::language]
    #[derive(Debug)]
    pub struct Program {
        pub rules: Vec<Declaration>,
    }

    #[derive(Debug, Clone)]
    pub enum Declaration {
        Input(#[rust_sitter::leaf(text = ".input")] (), Ident),
        Output(#[rust_sitter::leaf(text = ".output")] (), Ident),
        Rule(Rule),
    }

    #[derive(Debug, Clone)]
    pub struct Rule {
        pub target: Target,
        #[rust_sitter::leaf(text = ":-")]
        _from: (),
        #[rust_sitter::repeat(non_empty = true)]
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub sources: Vec<Target>,
        #[rust_sitter::leaf(text = ".")]
        _dot: Option<()>,
    }

    #[derive(Debug, Clone)]
    pub struct Target {
        pub name: Ident,
        #[rust_sitter::leaf(text = "(")]
        _l_paren: (),
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        fields: Vec<Ident>,
        #[rust_sitter::leaf(text = ")")]
        _r_paren: (),
    }

    #[derive(Debug, Clone)]
    pub struct Ident {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |s| s.to_string())]
        pub name: String,
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

use datalog_grammar::*;

fn gen_datalog_program(literal: proc_macro2::Literal, root: TokenStream) -> syn::Stmt {
    let str_node: syn::LitStr = parse_quote!(#literal);
    let actual_str = str_node.value();
    let program = datalog_grammar::parse(&actual_str).unwrap();

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

    for target in inputs {
        let target_ident = syn::Ident::new(&format!("{}_recv", &target.name), Span::call_site());
        flat_graph.add_statement(hydroflow_lang::parse::HfStatement::Named(
            NamedHfStatement {
                name: syn::Ident::new(&target.name, Span::call_site()),
                equals: Token![=](Span::call_site()),
                pipeline: Pipeline::Operator(Operator {
                    path: parse_quote!(recv_stream),
                    paren_token: Paren::default(),
                    args: vec![parse_quote!(#target_ident)]
                        .iter()
                        .cloned::<syn::Expr>()
                        .collect(),
                }),
            },
        ));
    }

    for target in outputs {
        flat_graph.add_statement(hydroflow_lang::parse::HfStatement::Named(
            NamedHfStatement {
                name: syn::Ident::new(&target.name, Span::call_site()),
                equals: Token![=](Span::call_site()),
                pipeline: Pipeline::Operator(Operator {
                    path: parse_quote!(for_each),
                    paren_token: Paren::default(),
                    args: vec![parse_quote!(|v| println!("{:?}", v))]
                        .iter()
                        .cloned::<syn::Expr>()
                        .collect(),
                }),
            },
        ));
    }

    for rule in rules {
        let target = rule.target.name;
        let target_ident = syn::Ident::new(&target.name, Span::call_site());
        let sources: Vec<Ident> = rule
            .sources
            .iter()
            .map(|source| source.name.clone())
            .collect();

        // TODO(shadaj): multiple sources
        flat_graph.add_statement(hydroflow_lang::parse::HfStatement::Pipeline(
            Pipeline::Link(PipelineLink {
                lhs: Box::new(Pipeline::Name(syn::Ident::new(
                    &sources[0].name,
                    Span::call_site(),
                ))),
                connector: ArrowConnector {
                    src: None,
                    arrow: Token![->](Span::call_site()),
                    dst: None,
                },
                rhs: Box::new(Pipeline::Name(target_ident)),
            }),
        ));
    }

    dbg!(flat_graph.surface_syntax_string());

    let code_tokens = flat_graph.into_partitioned_graph().as_code(root);

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

            out(x, y) :- in(x, y).
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
