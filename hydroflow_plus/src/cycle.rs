use std::marker::PhantomData;

use syn::parse_quote;

use crate::node::{HfDeploy, HfNode};
use crate::{HfBuilder, HfStream};

pub struct HfCycle<'a, T, D: HfDeploy<'a> + ?Sized, N: HfNode<'a, D>> {
    pub(crate) ident: syn::Ident,
    pub(crate) node: N,
    pub(crate) graph: &'a HfBuilder<'a, D>,
    pub(crate) _phantom: PhantomData<T>,
}

impl<'a, T, D: HfDeploy<'a>, N: HfNode<'a, D>> HfCycle<'a, T, D, N> {
    pub fn complete(self, stream: &HfStream<'a, T, D, N>) {
        let ident = self.ident;
        let stream_ident = stream.ident.clone();

        self.graph
            .builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #stream_ident -> #ident;
            });
    }
}
