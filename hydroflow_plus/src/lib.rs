use std::marker::PhantomData;

pub use hydroflow;
use hydroflow::scheduled::context::Context;
use hydroflow::scheduled::graph::Hydroflow;
use proc_macro2::TokenStream;
use quote::quote;
use stageleft::runtime_support::FreeVariable;
use stageleft::Quoted;

mod stream;
pub use stream::HfStream;

mod builder;
pub use builder::HfBuilder;

#[derive(Clone)]
pub struct RuntimeContext<'a> {
    _phantom: PhantomData<&'a mut &'a ()>,
}

impl Copy for RuntimeContext<'_> {}

impl<'a> FreeVariable<&'a Context> for RuntimeContext<'a> {
    fn to_tokens(self) -> (Option<TokenStream>, Option<TokenStream>) {
        (None, Some(quote!(&context)))
    }
}

pub struct HfBuilt<'a> {
    tokens: TokenStream,
    _phantom: PhantomData<&'a mut &'a ()>,
}

impl<'a> Quoted<Hydroflow<'a>> for HfBuilt<'a> {}

impl<'a> FreeVariable<Hydroflow<'a>> for HfBuilt<'a> {
    fn to_tokens(self) -> (Option<TokenStream>, Option<TokenStream>) {
        (None, Some(self.tokens))
    }
}
