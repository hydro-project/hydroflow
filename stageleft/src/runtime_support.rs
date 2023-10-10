use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use proc_macro2::TokenStream;
use quote::quote;

thread_local!(pub static CURRENT_FINAL_CRATE: RefCell<Option<&'static str>> = RefCell::new(None));

pub trait ParseFromLiteral {
    fn parse_from_literal(literal: &syn::Expr) -> Self;
}

impl ParseFromLiteral for bool {
    fn parse_from_literal(literal: &syn::Expr) -> Self {
        match literal {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Bool(lit_bool),
                ..
            }) => lit_bool.value(),
            _ => panic!("Expected literal"),
        }
    }
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
    path: String,
}

impl Type {
    pub fn new(def: &str) -> Type {
        Type {
            path: def.to_string(),
        }
    }
}

impl ToGlobalFreeVariableTokens for Type {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        let final_crate = CURRENT_FINAL_CRATE.with(|f| *f.borrow()).unwrap();

        let final_crate_path: syn::Path = syn::parse_str(final_crate).unwrap();
        let parsed: syn::Path = syn::parse_str(&self.path).unwrap();
        // drop the first element of parsed, which is its crate name
        let parsed = parsed.segments.iter().skip(1).collect::<Vec<_>>();
        let parsed = syn::Path {
            leading_colon: None,
            segments: syn::punctuated::Punctuated::from_iter(parsed.into_iter().cloned()),
        };
        (
            Some(quote!(use ::#final_crate_path::__flow::#parsed;)),
            None,
        )
    }
}

pub struct Import<T> {
    module_path: &'static str,
    path: &'static str,
    as_name: &'static str,
    _phantom: PhantomData<T>,
}

impl<T> Copy for Import<T> {}
impl<T> Clone for Import<T> {
    fn clone(&self) -> Self {
        Import {
            module_path: self.module_path,
            path: self.path,
            as_name: self.as_name,
            _phantom: PhantomData,
        }
    }
}

pub fn create_import<T>(
    module_path: &'static str,
    path: &'static str,
    as_name: &'static str,
    _unused_type_check: T,
) -> Import<T> {
    Import {
        module_path,
        path,
        as_name,
        _phantom: PhantomData,
    }
}

impl<T> ToFreeVariableTokens for Import<T> {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        let final_crate = CURRENT_FINAL_CRATE.with(|f| *f.borrow()).unwrap();
        let final_crate_path: syn::Path = syn::parse_str(final_crate).unwrap();

        let module_path = syn::parse_str::<syn::Path>(self.module_path).unwrap();
        let parsed = syn::parse_str::<syn::Path>(self.path).unwrap();
        let as_ident = syn::Ident::new(self.as_name, proc_macro2::Span::call_site());
        (
            Some(quote!(use #final_crate_path::#module_path::#parsed as #as_ident;)),
            None,
        )
    }
}

impl<T> FreeVariable<T> for Import<T> {}

impl<T> ToGlobalFreeVariableTokens for Import<T> {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        let final_crate = CURRENT_FINAL_CRATE.with(|f| *f.borrow()).unwrap();
        let final_crate_path: syn::Path = syn::parse_str(final_crate).unwrap();

        let module_path = syn::parse_str::<syn::Path>(self.module_path).unwrap();
        let parsed = syn::parse_str::<syn::Path>(self.path).unwrap();
        let as_ident = syn::Ident::new(self.as_name, proc_macro2::Span::call_site());
        (
            Some(quote!(use #final_crate_path::#module_path::#parsed as #as_ident;)),
            None,
        )
    }
}
