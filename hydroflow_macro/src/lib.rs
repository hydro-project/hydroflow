#![cfg_attr(
    feature = "diagnostics",
    feature(proc_macro_diagnostic, proc_macro_span, proc_macro_def_site)
)]
#![allow(clippy::explicit_auto_deref)]

use hydroflow_lang::diagnostic::{Diagnostic, Level};
use hydroflow_lang::graph::{build_hfcode, partition_graph, FlatGraphBuilder};
use hydroflow_lang::parse::HfCode;
use proc_macro2::{Ident, Literal, Span};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, Attribute, GenericParam, ItemEnum, LitStr, Variant,
    WherePredicate,
};

/// Create a Hydroflow instance using Hydroflow's custom "surface syntax."
///
/// For example usage, take a look at the [`surface_*` tests in the `tests` folder](https://github.com/hydro-project/hydroflow/tree/main/hydroflow/tests)
/// or the [`examples` folder](https://github.com/hydro-project/hydroflow/tree/main/hydroflow/examples)
/// in the [Hydroflow repo](https://github.com/hydro-project/hydroflow).
// TODO(mingwei): rustdoc examples inline.
#[proc_macro]
pub fn hydroflow_syntax(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    hydroflow_syntax_internal(input, Some(Level::Help))
}

/// [`hydroflow_syntax!`] but will not emit any diagnostics (errors, warnings, etc.).
///
/// Used for testing, users will want to use [`hydroflow_syntax!`] instead.
#[proc_macro]
pub fn hydroflow_syntax_noemit(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    hydroflow_syntax_internal(input, None)
}

fn root() -> proc_macro2::TokenStream {
    use std::env::{var as env_var, VarError};

    let hydroflow_crate = proc_macro_crate::crate_name("hydroflow")
        .expect("hydroflow should be present in `Cargo.toml`");
    match hydroflow_crate {
        proc_macro_crate::FoundCrate::Itself => {
            if Err(VarError::NotPresent) == env_var("CARGO_BIN_NAME")
                && Err(VarError::NotPresent) != env_var("CARGO_PRIMARY_PACKAGE")
                && Ok("hydroflow") == env_var("CARGO_CRATE_NAME").as_deref()
            {
                // In the crate itself, including unit tests.
                quote! { crate }
            } else {
                // In an integration test, example, bench, etc.
                quote! { ::hydroflow }
            }
        }
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident: Ident = Ident::new(&name, Span::call_site());
            quote! { ::#ident }
        }
    }
}

fn hydroflow_syntax_internal(
    input: proc_macro::TokenStream,
    min_diagnostic_level: Option<Level>,
) -> proc_macro::TokenStream {
    let macro_invocation_path = proc_macro::Span::call_site().source_file().path();

    let input = parse_macro_input!(input as HfCode);
    let root = root();
    let (graph_code_opt, diagnostics) = build_hfcode(input, &root, macro_invocation_path);
    let tokens = graph_code_opt
        .map(|(_graph, code)| code)
        .unwrap_or_else(|| quote! { #root::scheduled::graph::Hydroflow::new() });

    let diagnostics = diagnostics
        .iter()
        .filter(|diag: &&Diagnostic| Some(diag.level) <= min_diagnostic_level);

    #[cfg(feature = "diagnostics")]
    {
        diagnostics.for_each(Diagnostic::emit);
        tokens.into()
    }

    #[cfg(not(feature = "diagnostics"))]
    {
        let diagnostics = diagnostics.map(Diagnostic::to_tokens);
        quote! {
            {
                #(
                    #diagnostics
                )*
                #tokens
            }
        }
        .into()
    }
}

/// Parse Hydroflow "surface syntax" without emitting code.
///
/// Used for testing, users will want to use [`hydroflow_syntax!`] instead.
#[proc_macro]
pub fn hydroflow_parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let macro_invocation_path = proc_macro::Span::call_site().source_file().path();

    let input = parse_macro_input!(input as HfCode);

    let flat_graph_builder = FlatGraphBuilder::from_hfcode(input, macro_invocation_path);
    let (mut flat_graph, _uses, mut diagnostics) = flat_graph_builder.build();
    if !diagnostics.iter().any(Diagnostic::is_error) {
        if let Err(diagnostic) = flat_graph.merge_modules() {
            diagnostics.push(diagnostic);
        } else {
            let flat_mermaid = flat_graph.mermaid_string_flat();

            let part_graph = partition_graph(flat_graph).unwrap();
            let part_mermaid = part_graph.to_mermaid();

            let lit0 = Literal::string(&*flat_mermaid);
            let lit1 = Literal::string(&*part_mermaid);

            return quote! { println!("{}\n\n{}\n", #lit0, #lit1); }.into();
        }
    }

    diagnostics.iter().for_each(Diagnostic::emit);
    quote! {}.into()
}

#[doc(hidden)]
#[proc_macro]
pub fn surface_booktest_operators(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    assert!(input.is_empty(), "Input must be empty");
    let each = hydroflow_lang::graph::ops::OPERATORS.iter().map(|op| {
        let op_ident = Ident::new(op.name, Span::call_site());
        let op_filename = format!("../../docs/docgen/{}.md", op.name);
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

fn hydroflow_wrap(item: proc_macro::TokenStream, attribute: Attribute) -> proc_macro::TokenStream {
    use quote::ToTokens;

    let root = root();

    let mut input: syn::ItemFn = match syn::parse(item) {
        Ok(it) => it,
        Err(e) => return e.into_compile_error().into(),
    };

    let statements = input.block.stmts;

    input.block.stmts = parse_quote!(
        #root::tokio::task::LocalSet::new().run_until(async {
            #( #statements )*
        }).await
    );

    input.attrs.push(attribute);

    input.into_token_stream().into()
}

/// Checks that the given closure is a morphism. For now does nothing.
#[proc_macro]
pub fn morphism(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // TODO(mingwei): some sort of code analysis?
    item
}

/// Checks that the given closure is a monotonic function. For now does nothing.
#[proc_macro]
pub fn monotonic_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // TODO(mingwei): some sort of code analysis?
    item
}

#[proc_macro_attribute]
pub fn hydroflow_test(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let root = root();

    hydroflow_wrap(
        item,
        parse_quote!(
            #[#root::tokio::test(flavor = "current_thread")]
        ),
    )
}

#[proc_macro_attribute]
pub fn hydroflow_main(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let root = root();

    hydroflow_wrap(
        item,
        parse_quote!(
            #[#root::tokio::main(flavor = "current_thread")]
        ),
    )
}

#[proc_macro_derive(DemuxEnum)]
pub fn derive_answer_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let root = root();

    let ItemEnum {
        ident,
        generics,
        variants,
        ..
    } = parse_macro_input!(item as ItemEnum);

    // Sort variants alphabetically.
    let mut variants_sorted = variants.into_iter().collect::<Vec<_>>();
    variants_sorted.sort_by(|a, b| a.ident.cmp(&b.ident));

    let variant_pusherator_generics = variants_sorted
        .iter()
        .map(|variant| format_ident!("__Pusherator{}", variant.ident))
        .collect::<Vec<_>>();
    let variant_pusherator_localvars = variants_sorted
        .iter()
        .map(|variant| {
            format_ident!(
                "__pusherator_{}",
                variant.ident.to_string().to_lowercase(),
                span = variant.ident.span()
            )
        })
        .collect::<Vec<_>>();
    let variant_output_types = variants_sorted
        .iter()
        .map(|variant| match &variant.fields {
            syn::Fields::Named(fields) => {
                let field_types = fields.named.iter().map(|field| &field.ty);
                parse_quote! {
                    ( #( #field_types, )* )
                }
            }
            syn::Fields::Unnamed(fields) => {
                if 1 == fields.unnamed.len() {
                    fields.unnamed.first().unwrap().ty.clone()
                } else {
                    let field_types = fields.unnamed.iter().map(|field| &field.ty);
                    parse_quote! {
                        ( #( #field_types, )* )
                    }
                }
            }
            syn::Fields::Unit => parse_quote!(()),
        })
        .collect::<Vec<_>>();

    let mut full_generics = generics.clone();
    full_generics.params.extend(
        variant_pusherator_generics
            .iter()
            .map::<GenericParam, _>(|ident| parse_quote!(#ident)),
    );
    full_generics.make_where_clause().predicates.extend(
        variant_pusherator_generics
            .iter()
            .zip(variant_output_types.iter())
            .map::<WherePredicate, _>(|(pusherator_generic, output_type)| {
                parse_quote! {
                    #pusherator_generic: #root::pusherator::Pusherator<Item = #output_type>
                }
            }),
    );

    let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();
    let (impl_generics, _ty_generics, where_clause) = full_generics.split_for_impl();

    let variant_pats = variants_sorted
        .iter()
        .zip(variant_pusherator_localvars.iter())
        .map(|(variant, pushvar)| {
            let Variant { ident, fields, .. } = variant;
            let idents = fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    field
                        .ident
                        .clone()
                        .unwrap_or_else(|| format_ident!("_{}", i))
                })
                .collect::<Vec<_>>();
            let (fields_pat, push_item) = match fields {
                syn::Fields::Named(_) => {
                    (quote!( { #( #idents, )* } ), quote!( ( #( #idents, )* ) ))
                }
                syn::Fields::Unnamed(_) => {
                    (quote!( ( #( #idents ),* ) ), quote!( ( #( #idents ),* ) ))
                }
                syn::Fields::Unit => (quote!(), quote!(())),
            };
            quote! {
                Self::#ident #fields_pat => #pushvar.give(#push_item)
            }
        });

    quote! {
        impl #impl_generics #root::util::demux_enum::DemuxEnum<#root::variadics::var_type!( #( #variant_pusherator_generics, )* )>
            for #ident #ty_generics #where_clause
        {
            fn demux_enum(
                self,
                #root::variadics::var_args!( #( #variant_pusherator_localvars, )* ):
                    &mut #root::variadics::var_type!( #( #variant_pusherator_generics, )* )
            ) {
                match self {
                    #( #variant_pats, )*
                }
            }
        }
    }
    .into()
}
