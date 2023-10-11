use std::marker::PhantomData;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

pub mod internal {
    pub use proc_macro2::TokenStream;
    pub use quote::quote;
    pub use {proc_macro2, syn};
}

pub use stageleft_macro::{entry, q, quse_fn};

pub mod runtime_support;
use runtime_support::{FreeVariable, ToFreeVariableTokens, CURRENT_FINAL_CRATE};

pub trait QuotedContext {
    fn create() -> Self;
}

pub trait Quoted<T>: Sized + ToTokens {
    fn build(self) -> TokenStream {
        ToTokens::into_token_stream(self)
    }
}

type FreeVariables = Vec<(String, (Option<TokenStream>, Option<TokenStream>))>;

pub trait IntoQuotedOnce<'a, T>:
    FnOnce(&mut String, &mut String, &mut FreeVariables, bool) -> T + 'a
where
    Self: Sized,
{
    fn to_quoted(self) -> QuotedExpr<T> {
        let mut module_path = String::new();
        let mut str = String::new();
        let mut free_variables = Vec::new();
        // this is an uninit value so we can't drop it
        std::mem::forget(self(&mut module_path, &mut str, &mut free_variables, false));
        QuotedExpr::create(module_path, &str, free_variables)
    }
}

impl<'a, T, F: FnOnce(&mut String, &mut String, &mut FreeVariables, bool) -> T + 'a>
    IntoQuotedOnce<'a, T> for F
{
}

pub trait IntoQuotedMut<'a, T>:
    FnMut(&mut String, &mut String, &mut FreeVariables, bool) -> T + 'a
where
    Self: Sized,
{
    fn to_quoted(mut self) -> QuotedExpr<T> {
        let mut module_path = String::new();
        let mut str = String::new();
        let mut free_variables = Vec::new();
        // this is an uninit value so we can't drop it
        std::mem::forget(self(&mut module_path, &mut str, &mut free_variables, false));
        QuotedExpr::create(module_path, &str, free_variables)
    }
}

impl<'a, T, F: FnMut(&mut String, &mut String, &mut FreeVariables, bool) -> T + 'a>
    IntoQuotedMut<'a, T> for F
{
}

pub struct QuotedExpr<T> {
    module_path: syn::Path,
    expr: syn::Expr,
    free_variables: FreeVariables,
    _phantom: PhantomData<T>,
}

impl<T> QuotedExpr<T> {
    pub fn create(module_path: String, expr: &str, free_variables: FreeVariables) -> QuotedExpr<T> {
        let module_path = syn::parse_str(&module_path).unwrap();
        let expr = syn::parse_str(expr).unwrap();
        QuotedExpr {
            module_path,
            expr,
            free_variables,
            _phantom: PhantomData,
        }
    }
}

impl<T> Quoted<T> for QuotedExpr<T> {}

impl<T> ToTokens for QuotedExpr<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let instantiated_free_variables = self.free_variables.iter().flat_map(|(ident, value)| {
            let ident = syn::Ident::new(ident, Span::call_site());
            value.0.iter().map(|prelude| quote!(#prelude)).chain(
                value
                    .1
                    .iter()
                    .map(move |value| quote!(let #ident = #value;)),
            )
        });

        let final_crate_name = CURRENT_FINAL_CRATE.with(|f| *f.borrow()).unwrap();
        let final_crate = proc_macro_crate::crate_name(final_crate_name)
            .unwrap_or_else(|_| panic!("{final_crate_name} should be present in `Cargo.toml`"));
        let final_crate_root = match final_crate {
            proc_macro_crate::FoundCrate::Itself => syn::parse_str(final_crate_name).unwrap(),
            proc_macro_crate::FoundCrate::Name(name) => {
                let ident = syn::Ident::new(&name, Span::call_site());
                quote! { #ident }
            }
        };

        let module_path = &self.module_path;
        let mut module_path = module_path
            .segments
            .iter()
            .skip(1)
            .cloned()
            .collect::<Vec<_>>();
        module_path.insert(
            0,
            syn::PathSegment {
                ident: syn::Ident::new("__flow", Span::call_site()),
                arguments: syn::PathArguments::None,
            },
        );
        let module_path = syn::Path {
            leading_colon: None,
            segments: syn::punctuated::Punctuated::from_iter(module_path.into_iter()),
        };

        let expr = &self.expr;
        tokens.extend(quote!({
            use ::#final_crate_root::#module_path::*;
            #(#instantiated_free_variables)*
            #expr
        }));
    }
}

pub struct RuntimeData<T> {
    ident: &'static str,
    _phantom: PhantomData<T>,
}

impl<T: Copy> Copy for RuntimeData<T> {}

impl<T: Clone> Clone for RuntimeData<T> {
    fn clone(&self) -> Self {
        // TODO(shadaj): mark this as cloned so we clone it in the splice
        RuntimeData {
            ident: self.ident.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> RuntimeData<T> {
    pub fn new(ident: &'static str) -> RuntimeData<T> {
        RuntimeData {
            ident,
            _phantom: PhantomData,
        }
    }
}

impl<T> ToFreeVariableTokens for RuntimeData<T> {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        let ident = syn::Ident::new(self.ident, Span::call_site());
        (None, Some(quote!(#ident)))
    }
}

impl<T> FreeVariable<T> for RuntimeData<T> {}
