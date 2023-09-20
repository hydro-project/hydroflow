use std::collections::HashSet;

use proc_macro2::{Punct, Spacing, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, MacroDelimiter, PathArguments, Token, Type,
};

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

    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        for it in &node.attrs {
            self.visit_attribute(it);
        }
        if let Some(it) = &node.qself {
            self.visit_qself(it);
        }
        self.visit_path(&node.path);
        for el in Punctuated::pairs(&node.fields) {
            let it = el.value();
            self.visit_expr(&it.expr);
        }
        if let Some(it) = &node.rest {
            self.visit_expr(it);
        }
    }

    fn visit_expr_field(&mut self, i: &'ast syn::ExprField) {
        self.visit_expr(&i.base);
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

    let cloned_free_variables = visitor.free_variables.iter().map(|i| {
        let mut i_without_span = i.clone();
        i_without_span.set_span(Span::call_site());
        quote!(
            #[allow(non_upper_case_globals, non_snake_case)]
            let #i_without_span = ::std::clone::Clone::clone(&#i_without_span)
        )
    });

    let unitialized_free_variables = visitor.free_variables.iter().map(|i| {
        let mut i_without_span = i.clone();
        i_without_span.set_span(Span::call_site());
        quote!(
            #[allow(unused, non_upper_case_globals, non_snake_case)]
            let #i = ::#root::FreeVariable::uninitialized(&#i_without_span)
        )
    });

    let free_variables_vec = quote!(vec![#(#free_variables),*]);

    let expr_string = expr.clone().into_token_stream().to_string();
    proc_macro::TokenStream::from(quote!({
        #(#cloned_free_variables;)*
        move |set_str: &mut String, set_vec: &mut Vec<(String, (Option<#root::TokenStream>, Option<#root::TokenStream>))>, run: bool| {
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
                    use ::#(#prefix_unspanned::)*#name_ident_unspanned;
                    #[allow(non_upper_case_globals, non_snake_case)]
                    let #name_ident = #root::Import::create(
                        #full_path,
                        {
                            let __quse_local = ();
                            {
                                use ::#(#prefix::)*#name_ident as __quse_local;
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
                    use #(#prefix_unspanned::)*#name_ident_unspanned;
                    #[allow(non_upper_case_globals, non_snake_case)]
                    let #name_ident = #first.extend(
                        #suffix_full_path,
                        {
                            let __quse_local = ();
                            {
                                use #(#prefix::)*#name_ident as __quse_local;
                                __quse_local
                            }
                        }
                    );
                });
            } else {
                into.push(quote! {
                    #[allow(non_upper_case_globals, non_snake_case)]
                    let #name_ident = #root::Import::clone(&#name_ident);
                });
            }
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
        #(#all_paths_emitted;)*
    }
    .into()
}

#[proc_macro]
pub fn qtype(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let hydroflow_plus_crate = proc_macro_crate::crate_name("hydroflow_plus")
        .expect("hydroflow_plus should be present in `Cargo.toml`");
    let root = match hydroflow_plus_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { hydroflow_plus },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote! { #ident }
        }
    };

    let defn: syn::Item = parse_macro_input!(input as syn::Item);
    let name = match &defn {
        syn::Item::Struct(s) => &s.ident,
        syn::Item::Enum(e) => &e.ident,
        _ => panic!("qtype must be used on a struct or enum"),
    };

    let definition_string = defn.to_token_stream().to_string();

    quote! {
        #defn

        #[allow(non_upper_case_globals, non_snake_case)]
        fn #name() -> #root::Type {
            #root::Type::new(#definition_string)
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn flow(
    attr: proc_macro::TokenStream,
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

    let attr_params =
        syn::parse_macro_input!(attr with Punctuated<Type, Token![,]>::parse_terminated);

    let input = syn::parse_macro_input!(input as syn::ItemFn);
    let input_name = &input.sig.ident;

    let input_generics = &input.sig.generics;

    let mut runtime_data_params = Vec::new();
    let mut runtime_data_args = Vec::new();

    let param_parsing = input.sig.inputs.iter().skip(1).enumerate().map(|(i, input)| {
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
                    runtime_data_params.push(quote! {
                        #pat: #runtime_tpe
                    });
                    runtime_data_args.push(quote! {
                        ##pat
                    });
                    quote_spanned! {input.span()=>
                        let #pat: &#root::syn::Expr = &input_parsed[#i];
                    }
                } else {
                    quote_spanned! {input.span()=>
                        let #pat: #ty = #root::ParseFromLiteral::parse_from_literal(&input_parsed[#i]);
                    }
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
                #input_name #passed_generics(&#root::HydroflowContext::new(), #(#params_to_pass),*);
            });

            ::proc_macro::TokenStream::from(::#root::quote!({
                fn create_flow #input_generics(
                    #(#runtime_data_params),*
                ) -> #root::hydroflow::scheduled::graph::Hydroflow {
                    #pound dataflow_core
                }
                create_flow(#(#runtime_data_args),*)
            }))
        }
    })
}
