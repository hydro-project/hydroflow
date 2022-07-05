#![feature(proc_macro_diagnostic, proc_macro_span)]

use proc_macro2::{Literal, Span};
use quote::quote;
use syn::parse_macro_input;

use hydroflow_core::graph::flat_graph::FlatGraph;
use hydroflow_core::parse::HfCode;

#[proc_macro]
pub fn hydroflow_parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as HfCode);
    // // input.into_token_stream().into()

    let flat_graph = FlatGraph::from_hfcode(input);
    flat_graph.validate_operators();

    let flat_mermaid = flat_graph.mermaid_string();

    let part_graph = flat_graph.into_partitioned_graph();

    let part_mermaid = part_graph.mermaid_string();

    let lit0 = Literal::string(&*flat_mermaid);
    let lit1 = Literal::string(&*part_mermaid);

    quote! { println!("{}\n{}", #lit0, #lit1); }.into()
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
