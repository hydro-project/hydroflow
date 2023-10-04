use std::marker::PhantomData;
use std::mem::MaybeUninit;

use proc_macro2::TokenStream;
use quote::quote;

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
