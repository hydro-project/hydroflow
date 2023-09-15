use std::collections::HashSet;

use proc_macro2::{Punct, Spacing, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{MacroDelimiter, Type};

struct ScopeStack {
    scopes: Vec<HashSet<syn::Ident>>,
}

impl ScopeStack {
    fn new() -> ScopeStack {
        ScopeStack {
            scopes: vec![HashSet::new()],
        }
    }

    fn push(&mut self) {
        self.scopes.push(HashSet::new());
    }

    fn pop(&mut self) {
        self.scopes.pop();
    }

    fn insert(&mut self, ident: syn::Ident) {
        self.scopes
            .last_mut()
            .expect("Scope stack should not be empty")
            .insert(ident);
    }

    fn contains(&self, ident: &syn::Ident) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(ident))
    }
}

struct FreeVariableVisitor {
    free_variables: Vec<syn::Ident>,
    current_scope: ScopeStack,
}

fn is_prelude(ident: &syn::Ident) -> bool {
    let ident_str = ident.to_string();
    let prelude = vec![
        "Copy",
        "Send",
        "Sized",
        "Sync",
        "Unpin",
        "Drop",
        "Fn",
        "FnMut",
        "FnOnce",
        "drop",
        "Box",
        "ToOwned",
        "Clone",
        "PartialEq",
        "PartialOrd",
        "Eq",
        "Ord",
        "AsRef",
        "AsMut",
        "Into",
        "From",
        "Default",
        "Iterator",
        "Extend",
        "IntoIterator",
        "DoubleEndedIterator",
        "ExactSizeIterator",
        "Option",
        "Some",
        "None",
        "Result",
        "Ok",
        "Err",
        "String",
        "ToString",
        "Vec",
    ]
    .into_iter()
    .collect::<HashSet<&'static str>>();

    prelude.contains(&ident_str.as_str())
}

impl<'ast> Visit<'ast> for FreeVariableVisitor {
    fn visit_expr_closure(&mut self, i: &'ast syn::ExprClosure) {
        self.current_scope.push();
        i.inputs.iter().for_each(|input| match input {
            syn::Pat::Ident(pat_ident) => self.current_scope.insert(pat_ident.ident.clone()),
            syn::Pat::Type(pat_type) => match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => self.current_scope.insert(pat_ident.ident.clone()),
                _ => panic!("Closure parameters must be identifiers"),
            },
            _ => panic!("Closure parameters must be identifiers"),
        });

        syn::visit::visit_expr_closure(self, i);

        self.current_scope.pop();
    }

    fn visit_block(&mut self, i: &'ast syn::Block) {
        self.current_scope.push();
        syn::visit::visit_block(self, i);
        self.current_scope.pop();
    }

    fn visit_local(&mut self, i: &'ast syn::Local) {
        i.init.iter().for_each(|init| {
            syn::visit::visit_local_init(self, init);
        });

        match &i.pat {
            syn::Pat::Ident(pat_ident) => {
                self.current_scope.insert(pat_ident.ident.clone());
            }
            _ => panic!("Local variables must be identifiers"),
        }
    }

    fn visit_ident(&mut self, i: &'ast proc_macro2::Ident) {
        if !self.current_scope.contains(i) {
            self.free_variables.push(i.clone());
        }
    }

    fn visit_expr_path(&mut self, i: &'ast syn::ExprPath) {
        if i.path.leading_colon.is_none() && !is_prelude(&i.path.segments.first().unwrap().ident) {
            syn::visit::visit_path_segment(self, i.path.segments.first().unwrap());
        }
    }

    fn visit_type_path(&mut self, i: &'ast syn::TypePath) {
        if i.path.leading_colon.is_none() && !is_prelude(&i.path.segments.first().unwrap().ident) {
            syn::visit::visit_path_segment(self, i.path.segments.first().unwrap());
        }
    }

    fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
        syn::visit::visit_expr(self, &i.receiver);
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
        current_scope: ScopeStack::new(),
    };
    visitor.visit_expr(&expr);

    let free_variables = visitor.free_variables.iter().map(|i| {
        let ident = i.clone();
        let ident_str = ident.to_string();
        quote!((#ident_str.to_string(), ::#root::FreeVariable::to_tokens(&#ident)))
    });

    let unitialized_free_variables = visitor.free_variables.iter().map(|i| {
        let mut i_without_span = i.clone();
        i_without_span.set_span(Span::call_site());
        quote!(
            #[allow(non_upper_case_globals, non_snake_case)]
            let #i_without_span = ::#root::FreeVariable::uninitialized(&#i)
        )
    });

    let free_variables_vec = quote!(vec![#(#free_variables),*]);

    let expr_string = expr.clone().into_token_stream().to_string();
    proc_macro::TokenStream::from(
        quote!(|set_str: &mut String, set_vec: &mut Vec<(String, (Option<::hydroflow_plus::TokenStream>, Option<::hydroflow_plus::TokenStream>))>, run: bool| {
            *set_str = #expr_string.to_string();
            *set_vec = #free_variables_vec;

            if !run {
                unsafe {
                    return ::std::mem::MaybeUninit::uninit().assume_init();
                }
            }

            #[allow(unreachable_code)]
            {
                #(#unitialized_free_variables;)*
                #expr
            }
        }),
    )
}

fn gen_use_paths(
    root: TokenStream,
    mut prefix: Vec<syn::Ident>,
    tree: &syn::UseTree,
    into: &mut Vec<TokenStream>,
) {
    match &tree {
        syn::UseTree::Path(path) => {
            prefix.push(path.ident.clone());
            gen_use_paths(root, prefix, &path.tree, into);
        }
        syn::UseTree::Name(name) => {
            let name_ident = name.ident.clone();
            let mut name_ident_unspanned = name_ident.clone();
            name_ident_unspanned.set_span(Span::call_site());
            let full_path = quote!(#(#prefix::)*#name_ident).to_string();
            let prefix_unspanned = prefix
                .iter()
                .map(|i| {
                    let mut i = i.clone();
                    i.set_span(Span::call_site());
                    i
                })
                .collect::<Vec<_>>();
            into.push(quote! {
                use #(#prefix_unspanned::)*#name_ident_unspanned;
                #[allow(non_upper_case_globals, non_snake_case)]
                let #name_ident = #root::Import::create(
                    #full_path,
                    {
                        let __quse_local = ();
                        {
                            use #(#prefix::)*#name_ident as __quse_local;
                            __quse_local
                        }
                    }
                );
            });
        }
        _ => todo!(),
    }
}

#[proc_macro]
pub fn quse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let hydroflow_plus_crate = proc_macro_crate::crate_name("hydroflow_plus")
        .expect("hydroflow_plus should be present in `Cargo.toml`");
    let root = match hydroflow_plus_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { hydroflow_plus },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote! { #ident }
        }
    };

    let input_tokens = proc_macro2::TokenStream::from(input);
    let import: syn::ItemUse = syn::parse_quote!(use ::#input_tokens;);
    let mut all_paths_emitted = vec![];
    gen_use_paths(root, vec![], &import.tree, &mut all_paths_emitted);
    // let all_paths = match &import {
    //     syn::UseTree::Path(p) => {
    //         vec![p]
    //     }
    //     _ => todo!()
    // };

    // let all_paths_emitted = all_paths.iter().map(|path| {
    //     let last = path.
    //     quote! {
    //         use #import;
    //     }
    // }).collect::<Vec<_>>();

    quote! {
        #(#all_paths_emitted;)*
    }
    .into()
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
