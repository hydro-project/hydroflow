use std::collections::HashSet;

use proc_macro2::{Punct, Spacing, Span};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{MacroDelimiter, Type};

struct FreeVariableVisitor {
    free_variables: Vec<syn::Ident>,
    current_scope: HashSet<syn::Ident>,
}

impl<'ast> Visit<'ast> for FreeVariableVisitor {
    fn visit_expr_closure(&mut self, i: &'ast syn::ExprClosure) {
        let added_inputs = i
            .inputs
            .iter()
            .filter(|input| match input {
                syn::Pat::Ident(pat_ident) => self.current_scope.insert(pat_ident.ident.clone()),
                _ => panic!("Closure parameters must be identifiers"),
            })
            .collect::<Vec<_>>();

        syn::visit::visit_expr_closure(self, i);

        for input in added_inputs {
            match input {
                syn::Pat::Ident(pat_ident) => {
                    self.current_scope.remove(&pat_ident.ident);
                }
                _ => panic!("Closure parameters must be identifiers"),
            }
        }
    }

    fn visit_ident(&mut self, i: &'ast proc_macro2::Ident) {
        if !self.current_scope.contains(i) {
            self.free_variables.push(i.clone());
        }
    }

    fn visit_macro(&mut self, i: &'ast syn::Macro) {
        // TODO(shadaj): emit a warning if our guess at parsing fails
        match i.delimiter {
            MacroDelimiter::Paren(_binding_0) => i
                .parse_body_with(
                    syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
                )
                .ok()
                .iter()
                .flatten()
                .for_each(|expr| {
                    self.visit_expr(expr);
                }),
            MacroDelimiter::Brace(_binding_0) => i
                .parse_body_with(syn::Block::parse_within)
                .ok()
                .iter()
                .flatten()
                .for_each(|stmt| {
                    self.visit_stmt(stmt);
                }),
            MacroDelimiter::Bracket(_binding_0) => i
                .parse_body_with(
                    syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
                )
                .ok()
                .iter()
                .flatten()
                .for_each(|expr| {
                    self.visit_expr(expr);
                }),
        }
    }
}

#[proc_macro]
pub fn q(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let hydroflow_plus_crate = proc_macro_crate::crate_name("hydroflow_plus")
        .expect("hydroflow_plus should be present in `Cargo.toml`");
    let root = match hydroflow_plus_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { hydroflow_plus },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote! { #ident }
        }
    };

    let expr = syn::parse_macro_input!(input as syn::Expr);
    let mut visitor = FreeVariableVisitor {
        free_variables: Vec::new(),
        current_scope: HashSet::new(),
    };
    visitor.visit_expr(&expr);

    let free_variables = visitor.free_variables.iter().map(|i| {
        let ident = i.clone();
        let ident_str = ident.to_string();
        quote!((#ident_str.to_string(), ::#root::FreeVariable::to_tokens(&#ident)))
    });

    let unitialized_free_variables = visitor
        .free_variables
        .iter()
        .map(|i| quote!(let #i = ::#root::FreeVariable::uninitialized(&#i)));

    let free_variables_vec = quote!(vec![#(#free_variables),*]);

    let expr_string = expr.clone().into_token_stream().to_string();
    proc_macro::TokenStream::from(quote!(#root::QuotedExpr::create(
        #expr_string,
        #free_variables_vec,
        {
            #(#unitialized_free_variables;)*
            #expr
        }
    )))
}

#[proc_macro_attribute]
pub fn flow(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let hydroflow_plus_crate = proc_macro_crate::crate_name("hydroflow_plus")
        .expect("hydroflow_plus should be present in `Cargo.toml`");
    let root = match hydroflow_plus_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { hydroflow_plus },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote! { #ident }
        }
    };

    let input = syn::parse_macro_input!(input as syn::ItemFn);
    let input_name = &input.sig.ident;

    let mut runtime_data_stmts = Vec::new();

    let param_parsing = input.sig.inputs.iter().enumerate().map(|(i, input)| {
        match input {
            syn::FnArg::Receiver(_) => panic!("Flow functions cannot take self"),
            syn::FnArg::Typed(pat_type) => {
                let is_runtime = match pat_type.ty.as_ref() {
                    Type::Path(path) => {
                        path.path.segments.len() == 1 && path.path.segments[0].ident == "RuntimeData"
                    }
                    _ => false,
                };

                let pat = pat_type.pat.clone();
                let ty = pat_type.ty.clone();

                if is_runtime {
                    runtime_data_stmts.push(quote! {
                        let #pat = ##pat;
                    });
                    quote_spanned! {input.span()=>
                        let #pat = &input_parsed[#i];
                    }
                } else {
                    quote_spanned! {input.span()=>
                        let #pat: #ty = hydroflow_plus::ParseFromLiteral::parse_from_literal(&input_parsed[#i]);
                    }
                }
            }
        }
    });

    let params_to_pass = input.sig.inputs.iter().map(|input| match input {
        syn::FnArg::Receiver(_) => panic!("Flow functions cannot take self"),
        syn::FnArg::Typed(pat_type) => {
            let is_runtime = match pat_type.ty.as_ref() {
                Type::Path(path) => {
                    path.path.segments.len() == 1 && path.path.segments[0].ident == "RuntimeData"
                }
                _ => false,
            };

            if is_runtime {
                let pat_ident = match pat_type.pat.as_ref() {
                    syn::Pat::Ident(pat_ident) => pat_ident,
                    _ => panic!("RuntimeData must be an identifier"),
                };
                let pat_str = pat_ident.ident.to_string();
                quote!(#root::RuntimeData::new(#pat_str))
            } else {
                let pat = pat_type.pat.clone();
                quote!(#pat)
            }
        }
    });

    let expected_arg_count = input.sig.inputs.len();

    let pound = Punct::new('#', Spacing::Alone);

    proc_macro::TokenStream::from(quote_spanned! {input.span()=>
        #[proc_macro]
        pub fn #input_name(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            #[allow(unused)]
            let input_parsed = #root::syn::parse::Parser::parse(
                #root::syn::punctuated::Punctuated::<#root::syn::Expr, #root::syn::Token![,]>::parse_terminated,
                input
            ).unwrap();

            if input_parsed.len() != #expected_arg_count {
                panic!("Expected {} arguments, got {}", #expected_arg_count, input_parsed.len());
            }

            #(#param_parsing)*

            #input
            let dataflow_core = #root::hydroflow_build(|| {
                #input_name(#(#params_to_pass),*);
            });

            ::proc_macro::TokenStream::from(::#root::quote!({
                #(#runtime_data_stmts)*
                #pound dataflow_core
            }))
        }
    })
}
