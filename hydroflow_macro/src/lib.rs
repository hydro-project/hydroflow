#![feature(proc_macro_diagnostic, proc_macro_span)]
#![allow(clippy::explicit_auto_deref)]

use hydroflow_lang::graph::flat_to_partitioned::partition_graph;
use proc_macro2::{Ident, Literal, Span};
use quote::quote;
use syn::{parse_macro_input, LitStr};

use hydroflow_lang::diagnostic::Level;
use hydroflow_lang::graph::FlatGraphBuilder;
use hydroflow_lang::parse::HfCode;

#[proc_macro]
pub fn hydroflow_syntax(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let hydroflow_crate = proc_macro_crate::crate_name("hydroflow")
        .expect("hydroflow should be present in `Cargo.toml`");
    let root = match hydroflow_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { hydroflow },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { #ident }
        }
    };

    let input = parse_macro_input!(input as HfCode);

    let flat_graph_builder = FlatGraphBuilder::from_hfcode(input);
    let flat_graph_result = flat_graph_builder.build(Level::Help);
    if let Ok(flat_graph) = flat_graph_result {
        match partition_graph(flat_graph) {
            Ok(part_graph) => return part_graph.as_code(root, true).into(),
            Err(diagnostic) => diagnostic.emit(),
        }
    }
    quote! { #root::scheduled::graph::Hydroflow::new() }.into()
}

#[proc_macro]
pub fn hydroflow_parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as HfCode);

    let flat_graph_builder = FlatGraphBuilder::from_hfcode(input);
    let flat_graph = flat_graph_builder
        .build(Level::Help)
        .unwrap_or_else(std::convert::identity);
    let flat_mermaid = flat_graph.mermaid_string_flat();

    let part_graph = partition_graph(flat_graph).unwrap();
    let part_mermaid = part_graph.to_serde_graph().to_mermaid();

    let lit0 = Literal::string(&*flat_mermaid);
    let lit1 = Literal::string(&*part_mermaid);

    quote! { println!("{}\n\n{}\n", #lit0, #lit1); }.into()
}

#[doc(hidden)]
#[proc_macro]
pub fn surface_booktest_operators(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    assert!(input.is_empty(), "Input must be empty");
    let each = hydroflow_lang::graph::ops::OPERATORS.iter().map(|op| {
        let op_ident = Ident::new(op.name, Span::call_site());
        let op_filename = format!("../../book/docgen/{}.md", op.name);
        let lit_filename = LitStr::new(&*op_filename, Span::call_site());
        quote! {
            #[doc = include_str!(#lit_filename)]
            mod #op_ident {}
        }
    });
    let out = quote! {
        #( #each )*
    };
    out.into()
}
