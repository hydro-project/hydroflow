use std::marker::PhantomData;

use proc_macro2::{Span, TokenStream};
use quote::quote;

pub mod internal {
    pub use proc_macro2::TokenStream;
    pub use quote::quote;
    pub use {proc_macro2, syn};
}

pub use stageleft_macro::{entry, q, quse_fn};

pub mod runtime_support;
use runtime_support::{FreeVariable, CURRENT_FINAL_CRATE};

#[macro_export]
macro_rules! stageleft_crate {
    ($macro_crate:ident) => {
        #[doc(hidden)]
        pub use $macro_crate as __macro;

        #[doc(hidden)]
        pub mod __staged {
            include!(concat!(env!("OUT_DIR"), "/lib_pub.rs"));
        }
    };
}

pub trait QuotedContext {
    fn create() -> Self;
}

pub trait Quoted<T>: Sized {
    fn splice(self) -> TokenStream;
}

type FreeVariables = Vec<(String, (Option<TokenStream>, Option<TokenStream>))>;

pub trait IntoQuotedOnce<'a, T>:
    FnOnce(&mut String, &mut TokenStream, &mut FreeVariables, bool) -> T + 'a
where
    Self: Sized,
{
}

impl<'a, T, F: FnOnce(&mut String, &mut TokenStream, &mut FreeVariables, bool) -> T + 'a>
    IntoQuotedOnce<'a, T> for F
{
}

impl<'a, T, F: FnOnce(&mut String, &mut TokenStream, &mut FreeVariables, bool) -> T + 'a> Quoted<T>
    for F
{
    fn splice(self) -> TokenStream {
        let mut module_path = String::new();
        let mut expr_tokens = TokenStream::new();
        let mut free_variables = Vec::new();
        // this is an uninit value so we can't drop it
        std::mem::forget(self(
            &mut module_path,
            &mut expr_tokens,
            &mut free_variables,
            false,
        ));

        let instantiated_free_variables = free_variables.iter().flat_map(|(ident, value)| {
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

        let module_path: syn::Path = syn::parse_str(&module_path).unwrap();
        let module_path = module_path
            .segments
            .iter()
            .skip(1) // skip crate
            .cloned()
            .collect::<Vec<_>>();
        let module_path = syn::Path {
            leading_colon: None,
            segments: syn::punctuated::Punctuated::from_iter(module_path),
        };

        let expr: syn::Expr = syn::parse(expr_tokens.into()).unwrap();
        quote!({
            use ::#final_crate_root::#module_path::*;
            #(#instantiated_free_variables)*
            #expr
        })
    }
}

pub trait IntoQuotedMut<'a, T>:
    FnMut(&mut String, &mut TokenStream, &mut FreeVariables, bool) -> T + 'a
where
    Self: Sized,
{
}

impl<'a, T, F: FnMut(&mut String, &mut TokenStream, &mut FreeVariables, bool) -> T + 'a>
    IntoQuotedMut<'a, T> for F
{
}

pub struct RuntimeData<T> {
    ident: &'static str,
    _phantom: PhantomData<T>,
}

impl<T: Copy> Copy for RuntimeData<T> {}

// TODO(shadaj): relax this to allow for non-copy types
impl<T: Copy> Clone for RuntimeData<T> {
    fn clone(&self) -> Self {
        *self
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

impl<T> FreeVariable<T> for RuntimeData<T> {
    fn to_tokens(&self) -> (Option<TokenStream>, Option<TokenStream>) {
        let ident = syn::Ident::new(self.ident, Span::call_site());
        (None, Some(quote!(#ident)))
    }
}
