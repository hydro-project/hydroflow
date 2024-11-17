use std::marker::PhantomData;

use hydroflow::scheduled::context::Context;
use proc_macro2::TokenStream;
use quote::quote;
use stageleft::runtime_support::FreeVariable;

use crate::staging_util::Invariant;

#[derive(Clone)]
pub struct RuntimeContext<'a> {
    _phantom: Invariant<'a>,
}

impl RuntimeContext<'_> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl Copy for RuntimeContext<'_> {}

impl Default for RuntimeContext<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> FreeVariable<&'a Context> for RuntimeContext<'a> {
    fn to_tokens(self) -> (Option<TokenStream>, Option<TokenStream>) {
        (None, Some(quote!(&context)))
    }
}
