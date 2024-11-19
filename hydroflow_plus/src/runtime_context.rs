use std::marker::PhantomData;

use hydroflow::scheduled::context::Context;
use proc_macro2::TokenStream;
use quote::quote;
use stageleft::runtime_support::FreeVariableWithContext;

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

impl<'a, Ctx> FreeVariableWithContext<Ctx> for RuntimeContext<'a> {
    type O = &'a Context;

    fn to_tokens(self, _ctx: &Ctx) -> (Option<TokenStream>, Option<TokenStream>) {
        (None, Some(quote!(&context)))
    }
}
