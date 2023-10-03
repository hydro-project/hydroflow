use std::marker::PhantomData;
use std::mem::MaybeUninit;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

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

pub trait ParseFromLiteral {
    fn parse_from_literal(literal: &syn::Expr) -> Self;
}

impl ParseFromLiteral for u32 {
    fn parse_from_literal(literal: &syn::Expr) -> Self {
        match literal {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) => lit_int.base10_parse().unwrap(),
            _ => panic!("Expected literal"),
        }
    }
}

pub trait ToFreeVariableTokens {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>);
}

pub trait ToGlobalFreeVariableTokens {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>);
}

impl<T: ToGlobalFreeVariableTokens, F: Fn() -> T> ToFreeVariableTokens for F {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        let value = self();
        value.to_tokens()
    }
}

impl<T: ToGlobalFreeVariableTokens, F: Fn() -> T> FreeVariable<()> for F {}

pub trait FreeVariable<O>
where
    Self: Sized,
{
    fn uninitialized(self) -> O {
        #[allow(clippy::uninit_assumed_init)]
        unsafe {
            MaybeUninit::uninit().assume_init()
        }
    }
}

impl ToFreeVariableTokens for u32 {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        (None, Some(quote!(#self)))
    }
}

impl FreeVariable<u32> for u32 {}

impl ToGlobalFreeVariableTokens for u32 {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        (None, Some(quote!(#self)))
    }
}

pub struct RuntimeData<T> {
    ident: &'static str,
    _phantom: PhantomData<T>,
}

impl<T: Copy> Copy for RuntimeData<T> {}

impl<T> Clone for RuntimeData<T> {
    fn clone(&self) -> Self {
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

pub struct Import<T, P> {
    parent: Option<P>,
    path: &'static str,
    _phantom: PhantomData<T>,
}

impl<T, P: Copy> Copy for Import<T, P> {}
impl<T, P: Copy> Clone for Import<T, P> {
    fn clone(&self) -> Self {
        Import {
            parent: self.parent,
            path: self.path,
            _phantom: PhantomData,
        }
    }
}

pub fn create_import<T>(path: &'static str, _unused_type_check: T) -> Import<T, u32> {
    Import {
        parent: None,
        path,
        _phantom: PhantomData,
    }
}

impl<T, P: ToFreeVariableTokens + Copy> Import<T, P> {
    pub fn extend<T2>(
        &self,
        path: &'static str,
        _unused_type_check: T2,
    ) -> Import<T2, Import<T, P>> {
        Import {
            parent: Some(*self),
            path,
            _phantom: PhantomData,
        }
    }
}

impl<T, P: ToFreeVariableTokens> ToFreeVariableTokens for Import<T, P> {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        if let Some(parent) = &self.parent {
            let (prelude, value) = parent.to_tokens();
            let parsed = syn::parse_str::<syn::Path>(self.path).unwrap();
            (prelude, Some(quote!(#value::#parsed)))
        } else {
            let parsed = syn::parse_str::<syn::Path>(self.path).unwrap();
            (Some(quote!(use ::#parsed;)), None)
        }
    }
}

impl<T, P: ToFreeVariableTokens> FreeVariable<T> for Import<T, P> {}

impl<T, P: ToGlobalFreeVariableTokens + Copy> ToGlobalFreeVariableTokens for Import<T, P> {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        if let Some(parent) = &self.parent {
            let (prelude, value) = parent.to_tokens();
            let parsed = syn::parse_str::<syn::Path>(self.path).unwrap();
            (prelude, Some(quote!(#value::#parsed)))
        } else {
            let parsed = syn::parse_str::<syn::Path>(self.path).unwrap();
            (Some(quote!(use ::#parsed;)), None)
        }
    }
}

pub struct Type {
    definition: String,
}

impl Type {
    pub fn new(def: &str) -> Type {
        Type {
            definition: def.to_string(),
        }
    }
}

impl ToGlobalFreeVariableTokens for Type {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        let parsed: syn::Item = syn::parse_str(&self.definition).unwrap();
        (Some(quote!(#parsed)), None)
    }
}
