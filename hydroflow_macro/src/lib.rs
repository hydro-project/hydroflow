#![feature(proc_macro_diagnostic, proc_macro_span)]
#![allow(clippy::explicit_auto_deref)]

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Fields, ItemEnum};

use hydroflow_lang::graph::flat_graph::FlatGraph;
use hydroflow_lang::parse::HfCode;

fn root() -> TokenStream {
    let hydroflow_crate = proc_macro_crate::crate_name("hydroflow")
        .expect("hydroflow should be present in `Cargo.toml`");
    match hydroflow_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { hydroflow },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { #ident }
        }
    }
}

#[proc_macro]
pub fn hydroflow_syntax(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as HfCode);

    let root = root();

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

    let part_graph = flat_graph.into_partitioned_graph().unwrap();
    let part_mermaid = part_graph.to_serde_graph().to_mermaid();

    let lit0 = Literal::string(&*flat_mermaid);
    let lit1 = Literal::string(&*part_mermaid);

    quote! { println!("{}\n\n{}\n", #lit0, #lit1); }.into()
}

/// TODO(mingwei): move to separate variadic_list_macro package
#[proc_macro_derive(Split)]
pub fn derive_split(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let root = root();

    let ItemEnum {
        ident, variants, ..
    } = parse_macro_input!(input as ItemEnum);

    let variant_types = variants.iter().map(|v| {
        let field_types = v.fields.iter().map(|field| &field.ty);
        quote_spanned! {v.span()=>
            ( #( #field_types, )* )
        }
    });

    let none = &Ident::new("None", Span::call_site());
    let nones = vec![none; variants.len() - 1];

    let variant_matches = variants.iter().enumerate().map(|(i, v)| {
        let variant_name = &v.ident;
        let field_names = v.fields.iter().enumerate().map(|(j, field)| {
            field
                .ident
                .clone()
                .unwrap_or_else(|| Ident::new(&*format!("__{}", j), field.span()))
        });
        let fields = {
            let field_names = field_names.clone();
            match &v.fields {
                Fields::Named(_) => quote_spanned! {v.fields.span()=>
                    { #( #field_names, )* }
                },
                Fields::Unnamed(_) => quote_spanned! {v.fields.span()=>
                    ( #( #field_names, )* )
                },
                Fields::Unit => quote! {},
            }
        };

        let (nones_a, nones_b) = nones.split_at(i);
        quote_spanned! {v.span()=>
            Self::#variant_name #fields => #root::tl!(
                #( #nones_a, )*
                Some((#( #field_names, )*)),
                #( #nones_b, )*
            )
        }
    });

    // let variant_names = variants
    //     .iter()
    //     .map(|v| v.ident.to_string())
    //     .map(|name| name.from_case(Case::Pascal).to_case(Case::Snake))
    //     .map(|name| Ident::new(&*name, Span::call_site()));

    quote! {
        impl hydroflow::util::Split for #ident {
            type Split = #root::tt!(
                #( Option<#variant_types>, )*
            );
            fn split(self) -> Self::Split {
                match self {
                    #( #variant_matches, )*
                }
            }
        }
    }
    .into()
}
