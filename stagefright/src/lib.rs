use std::marker::PhantomData;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

pub mod internal {
    pub use proc_macro2::TokenStream;
    pub use quote::quote;
    pub use syn;
}

pub use stagefright_macro::{entry, q, qtype, quse, quse_type};

pub mod runtime_support;
use runtime_support::{FreeVariable, ToFreeVariableTokens};

pub trait QuotedContext {
    fn create() -> Self;
}

pub trait Quoted<T>: Sized + ToTokens {
    fn build(self) -> TokenStream {
        ToTokens::into_token_stream(self)
    }
}

type FreeVariables = Vec<(String, (Option<TokenStream>, Option<TokenStream>))>;

pub trait IntoQuotedOnce<'a, T>: FnOnce(&mut String, &mut FreeVariables, bool) -> T + 'a
where
    Self: Sized,
{
    fn to_quoted(self) -> QuotedExpr<T> {
        let mut str = String::new();
        let mut free_variables = Vec::new();
        // this is an uninit value so we can't drop it
        std::mem::forget(self(&mut str, &mut free_variables, false));
        QuotedExpr::create(&str, free_variables)
    }
}

impl<'a, T, F: FnOnce(&mut String, &mut FreeVariables, bool) -> T + 'a> IntoQuotedOnce<'a, T>
    for F
{
}

pub trait IntoQuotedMut<'a, T>: FnMut(&mut String, &mut FreeVariables, bool) -> T + 'a
where
    Self: Sized,
{
    fn to_quoted(mut self) -> QuotedExpr<T> {
        let mut str = String::new();
        let mut free_variables = Vec::new();
        // this is an uninit value so we can't drop it
        std::mem::forget(self(&mut str, &mut free_variables, false));
        QuotedExpr::create(&str, free_variables)
    }
}

impl<'a, T, F: FnMut(&mut String, &mut FreeVariables, bool) -> T + 'a> IntoQuotedMut<'a, T> for F {}

pub struct QuotedExpr<T> {
    expr: syn::Expr,
    free_variables: FreeVariables,
    _phantom: PhantomData<T>,
}

impl<T> QuotedExpr<T> {
    pub fn create(expr: &str, free_variables: FreeVariables) -> QuotedExpr<T> {
        let expr = syn::parse_str(expr).unwrap();
        QuotedExpr {
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

        let expr = &self.expr;
        tokens.extend(quote!({
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
