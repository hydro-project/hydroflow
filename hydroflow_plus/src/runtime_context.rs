use std::marker::PhantomData;

use hydroflow::scheduled::context::Context;
use proc_macro2::TokenStream;
use quote::quote;
use stageleft::runtime_support::FreeVariable;

#[derive(Clone)]
pub struct RuntimeContext<'a> {
    _phantom: PhantomData<&'a mut &'a ()>,
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
