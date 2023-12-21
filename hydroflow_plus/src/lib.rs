use std::marker::PhantomData;

use hydroflow::scheduled::context::Context;
pub use hydroflow::scheduled::graph::Hydroflow;
pub use hydroflow::*;
use proc_macro2::TokenStream;
use quote::quote;
use stageleft::runtime_support::FreeVariable;
use stageleft::Quoted;

pub mod runtime_support {
    pub use bincode;
}

mod stream;
pub use stream::Stream;

pub mod node;

pub mod cycle;
pub use cycle::HfCycle;

pub mod builder;
pub use builder::GraphBuilder;

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

impl<'a> Quoted<'a, Hydroflow<'a>> for HfBuilt<'a> {}

impl<'a> FreeVariable<Hydroflow<'a>> for HfBuilt<'a> {
    fn to_tokens(self) -> (Option<TokenStream>, Option<TokenStream>) {
        (None, Some(self.tokens))
    }
}
