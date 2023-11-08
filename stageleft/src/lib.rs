use core::panic;
use std::marker::PhantomData;

use internal::CaptureVec;
use proc_macro2::{Span, TokenStream};
use quote::quote;

pub mod internal {
    pub use proc_macro2::{Span, TokenStream};
    pub use quote::quote;
    pub use {proc_macro2, proc_macro_crate, syn};

    pub type CaptureVec = Vec<(String, (Option<TokenStream>, Option<TokenStream>))>;
}

pub use stageleft_macro::{entry, q, quse_fn, runtime};

pub mod runtime_support;
use runtime_support::FreeVariable;

#[macro_export]
macro_rules! stageleft_crate {
    ($macro_crate:ident) => {
        #[cfg(not(feature = "macro"))]
        #[doc(hidden)]
        pub use $macro_crate as __macro;

        #[cfg(not(feature = "macro"))]
        #[doc(hidden)]
        #[allow(unused)]
        pub mod __staged {
            include!(concat!(env!("OUT_DIR"), "/lib_pub.rs"));
        }
    };
}

#[macro_export]
macro_rules! stageleft_macro_crate {
    () => {
        include!(concat!(env!("OUT_DIR"), "/lib.rs"));
    };
}

pub trait QuotedContext {
    fn create() -> Self;
}

impl QuotedContext for () {
    fn create() -> Self {}
}

pub trait Quoted<'a, T>: FreeVariable<T> {
    fn splice(self) -> TokenStream
    where
        Self: Sized,
    {
        let (prelude, value) = self.to_tokens();
        if prelude.is_some() {
            panic!("Quoted value should not have prelude");
        }

        value.unwrap()
    }
}

pub trait IntoQuotedOnce<'a, T>:
    FnOnce(&mut String, &mut &'static str, &mut TokenStream, &mut CaptureVec, bool) -> T
    + 'a
    + Quoted<'a, T>
{
    fn boxed(self) -> Box<dyn IntoQuotedOnce<'a, T>>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl<
        'a,
        T,
        F: FnOnce(&mut String, &mut &'static str, &mut TokenStream, &mut CaptureVec, bool) -> T + 'a,
    > Quoted<'a, T> for F
{
}

impl<
        'a,
        T,
        F: FnOnce(&mut String, &mut &'static str, &mut TokenStream, &mut CaptureVec, bool) -> T + 'a,
    > IntoQuotedOnce<'a, T> for F
{
}

impl<
        'a,
        T,
        F: FnOnce(&mut String, &mut &'static str, &mut TokenStream, &mut CaptureVec, bool) -> T + 'a,
    > FreeVariable<T> for F
{
    fn to_tokens(self) -> (Option<TokenStream>, Option<TokenStream>) {
        let mut module_path = String::new();
        let mut crate_name = "";
        let mut expr_tokens = TokenStream::new();
        let mut free_variables = Vec::new();
        // this is an uninit value so we can't drop it
        std::mem::forget(self(
            &mut module_path,
            &mut crate_name,
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

        let final_crate = proc_macro_crate::crate_name(crate_name)
            .unwrap_or_else(|_| panic!("{crate_name} should be present in `Cargo.toml`"));
        let final_crate_root = match final_crate {
            proc_macro_crate::FoundCrate::Itself => quote!(crate),
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
        (
            None,
            Some(quote!({
                use #final_crate_root::#module_path::*;
                #(#instantiated_free_variables)*
                #expr
            })),
        )
    }
}

pub trait IntoQuotedMut<'a, T>:
    FnMut(&mut String, &mut &'static str, &mut TokenStream, &mut CaptureVec, bool) -> T + 'a
{
}

impl<
        'a,
        T,
        F: FnMut(&mut String, &mut &'static str, &mut TokenStream, &mut CaptureVec, bool) -> T + 'a,
    > IntoQuotedMut<'a, T> for F
{
}

/// Represents a piece of data that will be passed into the generated code
pub struct RuntimeData<T> {
    ident: &'static str,
    _phantom: PhantomData<T>,
}

impl<'a, T> Quoted<'a, T> for RuntimeData<T> {}

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
    fn to_tokens(self) -> (Option<TokenStream>, Option<TokenStream>) {
        let ident = syn::Ident::new(self.ident, Span::call_site());
        (None, Some(quote!(#ident)))
    }
}
