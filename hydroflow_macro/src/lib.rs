#![feature(proc_macro_diagnostic, proc_macro_span)]
#![allow(clippy::explicit_auto_deref)]

use proc_macro2::{Ident, Literal, Span};
use quote::quote;
use syn::parse_macro_input;

use hydroflow_lang::graph::flat_graph::FlatGraph;
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

    let flat_graph = FlatGraph::from_hfcode(input);
    if !flat_graph.emit_operator_errors() {
        if let Ok(part_graph) = flat_graph.into_partitioned_graph() {
            return part_graph.as_code(root).into();
        }
    }
    quote! { #root::scheduled::graph::Hydroflow::new() }.into()
}

#[proc_macro]
pub fn hydroflow_parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as HfCode);
    // // input.into_token_stream().into()

    let flat_graph = FlatGraph::from_hfcode(input);
    flat_graph.emit_operator_errors();

    let flat_mermaid = flat_graph.mermaid_string();

    // let part_graph = flat_graph.into_partitioned_graph();
    // let part_mermaid = part_graph.to_serde_graph();

    let lit0 = Literal::string(&*flat_mermaid);
    // let lit1 = Literal::string(&*part_mermaid);

    quote! { println!("{}", #lit0); }.into()
}
