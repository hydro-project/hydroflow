use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use proc_macro2::{Span, TokenStream};
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

pub trait FreeVariable<O>
where
    Self: Sized,
{
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>);

    fn uninitialized(self) -> O {
        #[allow(clippy::uninit_assumed_init)]
        unsafe {
            MaybeUninit::uninit().assume_init()
        }
    }
}

impl FreeVariable<u32> for u32 {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        (None, Some(quote!(#self)))
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
        *self
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

impl<T> FreeVariable<T> for Import<T> {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
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

        let module_path = syn::parse_str::<syn::Path>(self.module_path).unwrap();
        let parsed = syn::parse_str::<syn::Path>(self.path).unwrap();
        let as_ident = syn::Ident::new(self.as_name, proc_macro2::Span::call_site());
        (
            Some(quote!(use #final_crate_root::#module_path::#parsed as #as_ident;)),
            None,
        )
    }
}
