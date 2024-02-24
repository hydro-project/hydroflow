use std::cell::RefCell;
use std::marker::PhantomData;

use syn::parse_quote;

use crate::__staged::builder::HfPlusNode;
use crate::builder::Builders;
use crate::location::Location;
use crate::Stream;

/// Represents a fixpoint cycle in the graph that will be fulfilled
/// by a stream that is not yet known.
///
/// See [`Stream`] for an explainer on the type parameters.
pub struct HfCycle<'a, T, W, N: Location<'a>> {
    pub(crate) ident: syn::Ident,
    pub(crate) node: N,
    pub(crate) builders: &'a Builders,
    pub(crate) ir_leaves: &'a RefCell<Vec<HfPlusNode>>,
    pub(crate) _phantom: PhantomData<(T, W)>,
}

impl<'a, T, W, N: Location<'a>> HfCycle<'a, T, W, N> {
    pub fn complete(self, stream: Stream<'a, T, W, N>) {
        let ident = self.ident;
        let concrete = stream.ensure_concrete();
        // TODO(shadaj): avoid having to concretize within cycles
        let stream_ident = concrete.ident.clone();

        self.builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.node.id())
            .or_default()
            .add_statement(parse_quote! {
                #stream_ident -> #ident;
            });

        self.ir_leaves.borrow_mut().push(HfPlusNode::CycleSink {
            ident,
            input: Box::new(concrete.ir_node.into_inner()),
        });
    }
}
