use std::marker::PhantomData;

use syn::parse_quote;

use crate::builder::Builders;
use crate::node::HfNode;
use crate::HfStream;

pub struct HfCycle<'a, T, W, N: HfNode<'a>> {
    pub(crate) ident: syn::Ident,
    pub(crate) node: N,
    pub(crate) builders: &'a Builders,
    pub(crate) _phantom: PhantomData<(T, W)>,
}

impl<'a, T, W, N: HfNode<'a>> HfCycle<'a, T, W, N> {
    pub fn complete(self, stream: &HfStream<'a, T, W, N>) {
        let ident = self.ident;
        let stream_ident = stream.ident.clone();

        self.builders
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
