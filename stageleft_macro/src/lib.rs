#![feature(proc_macro_span)]

use proc_macro2::{Punct, Spacing, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{AngleBracketedGenericArguments, PathArguments, Token, Type};

mod free_variable;
use free_variable::*;

#[proc_macro]
pub fn q(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stageleft_crate = proc_macro_crate::crate_name("stageleft")
        .expect("stageleft should be present in `Cargo.toml`");
    let root = match stageleft_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { stageleft },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote! { #ident }
        }
    };

    let expr = syn::parse_macro_input!(input as syn::Expr);
    let mut visitor = FreeVariableVisitor::default();
    visitor.visit_expr(&expr);

    let free_variables = visitor.free_variables.iter().map(|i| {
        let ident = i.clone();
        let ident_str = ident.to_string();
        quote!((#ident_str.to_string(), ::#root::runtime_support::FreeVariable::to_tokens(&#ident)))
    });

    let cloned_free_variables = visitor.free_variables.iter().map(|i| {
        let mut i_without_span = i.clone();
        i_without_span.set_span(Span::call_site());
        quote!(
            #[allow(non_upper_case_globals, non_snake_case)]
            let #i_without_span = #i_without_span;
        )
    });

    let unitialized_free_variables = visitor.free_variables.iter().map(|i| {
        let mut i_without_span = i.clone();
        i_without_span.set_span(Span::call_site());
        quote!(
            #[allow(unused, non_upper_case_globals, non_snake_case)]
            let #i = ::#root::runtime_support::FreeVariable::uninitialized(#i_without_span)
        )
    });

    let free_variables_vec = quote!(vec![#(#free_variables),*]);

    proc_macro::TokenStream::from(quote!({
        #(#cloned_free_variables;)*
        move |set_mod: &mut String, set_tokens: &mut #root::internal::TokenStream, set_vec: &mut Vec<(String, (Option<#root::internal::TokenStream>, Option<#root::internal::TokenStream>))>, run: bool| {
            *set_mod = module_path!().to_string();
            *set_tokens = #root::internal::quote! {
                #expr
            };
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
        }
    }))
}

fn gen_use_paths(
    root: TokenStream,
    is_rooted: bool,
    mut prefix: Vec<syn::Ident>,
    tree: &syn::UseTree,
    into: &mut Vec<TokenStream>,
) {
    match &tree {
        syn::UseTree::Path(path) => {
            prefix.push(path.ident.clone());
            gen_use_paths(root, is_rooted, prefix, &path.tree, into);
        }
        syn::UseTree::Group(group) => {
            for tree in &group.items {
                gen_use_paths(root.clone(), is_rooted, prefix.clone(), tree, into);
            }
        }
        syn::UseTree::Name(name) => {
            let name_ident = name.ident.clone();
            let mut name_ident_unspanned = name_ident.clone();
            name_ident_unspanned.set_span(Span::call_site());
            let prefix_unspanned = prefix
                .iter()
                .map(|i| {
                    let mut i = i.clone();
                    i.set_span(Span::call_site());
                    i
                })
                .collect::<Vec<_>>();

            if is_rooted {
                let full_path = quote!(#(#prefix::)*#name_ident).to_string();

                into.push(quote! {
                    #[allow(non_upper_case_globals, non_snake_case)]
                    let #name_ident_unspanned = #root::runtime_support::create_import(
                        #full_path,
                        {
                            let __quse_local = ();
                            {
                                use ::#(#prefix_unspanned::)*#name_ident_unspanned as __quse_local;
                                __quse_local
                            }
                        }
                    );
                });
            } else if !prefix.is_empty() {
                let first = prefix.first().unwrap();
                let prefix_suffix = prefix.iter().skip(1);
                let suffix_full_path = quote!(#(#prefix_suffix::)*#name_ident).to_string();

                into.push(quote! {
                    #[allow(non_upper_case_globals, non_snake_case)]
                    let #name_ident_unspanned = #first.extend(
                        #suffix_full_path,
                        {
                            let __quse_local = ();
                            {
                                use #(#prefix_unspanned::)*#name_ident_unspanned as __quse_local;
                                __quse_local
                            }
                        }
                    );
                });
            } else {
                into.push(quote! {
                    #[allow(non_upper_case_globals, non_snake_case)]
                    let #name_ident = #root::runtime_support::Import::clone(&#name_ident);
                });
            }
        }
        _ => todo!(),
    }
}

#[proc_macro]
pub fn quse_fn(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stageleft_crate = proc_macro_crate::crate_name("stageleft")
        .expect("stageleft should be present in `Cargo.toml`");
    let root = match stageleft_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { stageleft },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote! { #ident }
        }
    };

    let input_tokens = proc_macro2::TokenStream::from(input);
    let import: syn::ItemUse = syn::parse_quote!(use #input_tokens;);
    let mut all_paths_emitted = vec![];
    gen_use_paths(
        root,
        import.leading_colon.is_some(),
        vec![],
        &import.tree,
        &mut all_paths_emitted,
    );

    quote! {
        use #input_tokens;
        #(#all_paths_emitted;)*
    }
    .into()
}

#[proc_macro_attribute]
pub fn entry(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let stageleft_crate = proc_macro_crate::crate_name("stageleft")
        .expect("stageleft should be present in `Cargo.toml`");
    let root = match stageleft_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { stageleft },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote! { #ident }
        }
    };

    let attr_params =
        syn::parse_macro_input!(attr with Punctuated<Type, Token![,]>::parse_terminated);

    let input = syn::parse_macro_input!(input as syn::ItemFn);
    let input_name = &input.sig.ident;

    let input_generics = &input.sig.generics;

    let mut runtime_data_params = Vec::new();
    let mut runtime_data_args = Vec::new();

    let param_parsing = input.sig.inputs.iter().skip(1).enumerate().flat_map(|(i, input)| {
        match input {
            syn::FnArg::Receiver(_) => panic!("Flow functions cannot take self"),
            syn::FnArg::Typed(pat_type) => {
                let runtime_tpe = match pat_type.ty.as_ref() {
                    Type::Path(path) => {
                        if path.path.segments.len() == 1 && path.path.segments[0].ident == "RuntimeData" {
                            match &path.path.segments[0].arguments {
                                PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                                    args,
                                    ..
                                }) => Some(args[0].clone()),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                let pat = pat_type.pat.clone();
                let ty = pat_type.ty.clone();

                if let Some(runtime_tpe) = runtime_tpe {
                    let mut visitor = FreeVariableVisitor::default();

                    visitor.current_scope.insert_type(syn::Ident::new("RuntimeData", Span::call_site()));

                    visitor.visit_generics(input_generics);
                    visitor.visit_generic_argument(&runtime_tpe);

                    let mut out = vec![];

                    visitor.free_variables.iter().for_each(|i| {
                        let mut i_unspanned = i.clone();
                        i_unspanned.set_span(Span::call_site());
                        out.push(quote! {
                            if let Some(prelude) = ::#root::runtime_support::FreeVariable::to_tokens(&#i_unspanned).0 {
                                runtime_data_prelude.push(prelude);
                            }
                        });
                    });

                    runtime_data_params.push(quote! {
                        #pat: #runtime_tpe
                    });
                    runtime_data_args.push(quote! {
                        ##pat
                    });

                    out.push(quote_spanned! {input.span()=>
                        let #pat: &#root::internal::syn::Expr = &input_parsed[#i];
                    });

                    out
                } else {
                    vec![quote_spanned! {input.span()=>
                        let #pat: #ty = #root::runtime_support::ParseFromLiteral::parse_from_literal(&input_parsed[#i]);
                    }]
                }
            }
        }
    });

    let params_to_pass = input.sig.inputs.iter().skip(1).map(|input| match input {
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

    let expected_arg_count = input.sig.inputs.len() - 1;

    let pound = Punct::new('#', Spacing::Alone);
    let passed_generics = if attr_params.is_empty() {
        quote!()
    } else {
        quote!(::<#attr_params>)
    };

    // the return type is always of form `impl Quoted<T>`, this grabs `T` and any free variable imports for it
    let (return_type_free, return_type_inner) = match &input.sig.output {
        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::ImplTrait(impl_trait) => match impl_trait.bounds.first().unwrap() {
                syn::TypeParamBound::Trait(quoted_path) => {
                    match &quoted_path.path.segments[0].arguments {
                        syn::PathArguments::AngleBracketed(args) => {
                            match args.args.first().unwrap() {
                                syn::GenericArgument::Type(ty) => {
                                    let mut visitor = FreeVariableVisitor::default();

                                    visitor.visit_generics(input_generics);
                                    visitor.visit_type(ty);

                                    let mut out = vec![];

                                    visitor.free_variables.iter().for_each(|i| {
                                        let mut i_unspanned = i.clone();
                                        i_unspanned.set_span(Span::call_site());
                                        out.push(quote! {
                                            if let Some(prelude) = ::#root::runtime_support::FreeVariable::to_tokens(&#i_unspanned).0 {
                                                runtime_data_prelude.push(prelude);
                                            }
                                        });
                                    });

                                    (out, ty.clone())
                                }
                                _ => panic!(),
                            }
                        }
                        _ => panic!(),
                    }
                }
                _ => panic!(),
            },
            _ => panic!(),
        },
        _ => panic!(),
    };

    let orig_visibility = input.vis.clone();

    let input_contents = input
        .block
        .to_token_stream()
        .to_string()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();
    let input_hash = "macro_".to_string() + &sha256::digest(input_contents);
    let input_hash_ident = syn::Ident::new(&input_hash, Span::call_site());

    proc_macro::TokenStream::from(quote_spanned! {input.span()=>
        #[cfg(not(feature = "final"))]
        #orig_visibility fn #input_name(input: #root::internal::TokenStream) -> #root::internal::TokenStream {
            #[allow(unused)]
            let input_parsed = #root::internal::syn::parse::Parser::parse(
                #root::internal::syn::punctuated::Punctuated::<#root::internal::syn::Expr, #root::internal::syn::Token![,]>::parse_terminated,
                input.into()
            ).unwrap();

            if input_parsed.len() != #expected_arg_count {
                panic!("Expected {} arguments, got {}", #expected_arg_count, input_parsed.len());
            }

            #[allow(unused_mut)]
            let mut runtime_data_prelude = ::std::vec::Vec::<#root::internal::TokenStream>::new();

            #(#param_parsing)*
            #(#return_type_free)*

            #input
            let output_core = {
                let graph = #root::QuotedContext::create();
                #root::Quoted::splice(#input_name #passed_generics(&graph, #(#params_to_pass),*))
            };

            let final_crate = #root::runtime_support::CURRENT_FINAL_CRATE.with(|f| *f.borrow()).unwrap();
            let final_crate_path: #root::internal::syn::Path = #root::internal::syn::parse_str(final_crate).unwrap();

            let module_path: #root::internal::syn::Path = #root::internal::syn::parse_str(module_path!()).unwrap();
            let module_path = module_path.segments.iter().skip(1).cloned().collect::<Vec<_>>();
            let module_path = #root::internal::syn::Path {
                leading_colon: None,
                segments: #root::internal::syn::punctuated::Punctuated::from_iter(module_path.into_iter()),
            };

            ::#root::internal::quote!({
                use #pound final_crate_path :: #pound module_path :: *;
                #pound (#pound runtime_data_prelude)*
                fn create_flow #input_generics(
                    #(#runtime_data_params),*
                ) -> #return_type_inner {
                    #pound output_core
                }
                create_flow(#(#runtime_data_args),*)
            })
        }

        #[cfg(feature = "final")]
        pub use crate::__macro::#input_hash_ident as #input_name;
    })
}
