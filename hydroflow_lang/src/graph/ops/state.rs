use quote::{quote_spanned, ToTokens};
use syn::parse_quote;

use super::{
    OperatorCategory, OperatorConstraints, OperatorWriteOutput, PortListSpec, WriteContextArgs,
    RANGE_0, RANGE_1,
};
use crate::graph::{GraphEdgeType, OpInstGenerics, OperatorInstance, PortIndexValue};

// TODO(mingwei)
pub const STATE: OperatorConstraints = OperatorConstraints {
    name: "state",
    categories: &[OperatorCategory::Persistence],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: &(2..=2),
    soft_range_out: &(2..=2),
    num_args: 0,
    persistence_args: RANGE_0, // TODO(mingwei)?
    type_args: &(0..=1),
    is_external_input: false,
    ports_inn: None,
    ports_out: Some(|| PortListSpec::Fixed(parse_quote! { items, state })),
    input_delaytype_fn: |_| None,
    input_edgetype_fn: |_| Some(GraphEdgeType::Value),
    output_edgetype_fn: |idx| match idx {
        PortIndexValue::Path(path) if "state" == path.to_token_stream().to_string() => {
            GraphEdgeType::Reference
        }
        _else => GraphEdgeType::Value,
    },
    flow_prop_fn: None,
    write_fn: |&WriteContextArgs {
                   root,
                   context,
                   hydroflow,
                   op_span,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   op_inst:
                       OperatorInstance {
                           generics: OpInstGenerics { type_args, .. },
                           ..
                       },
                   ..
               },
               _diagnostics| {
        let lattice_type = type_args
            .first()
            .map(quote::ToTokens::to_token_stream)
            .unwrap_or(quote_spanned!(op_span=> _));

        let state_ident = &outputs[1];

        let write_prologue = quote_spanned! {op_span=>
            let #state_ident = #hydroflow.add_state(::std::cell::RefCell::new(
                <#lattice_type as ::std::default::Default>::default()
            ));
        };

        let write_iterator = if is_pull {
            let input = &inputs[0];
            quote_spanned! {op_span=>
                let #ident = {
                    fn check_input<'a, Item, Iter, Lat>(
                        iter: Iter,
                        state_handle: #root::scheduled::state::StateHandle<::std::cell::RefCell<Lat>>,
                        context: &'a #root::scheduled::context::Context,
                    ) -> impl 'a + ::std::iter::Iterator<Item = Item>
                    where
                        Item: ::std::clone::Clone,
                        Iter: 'a + ::std::iter::Iterator<Item = Item>,
                        Lat: 'static + #root::lattices::Merge<Item>,
                    {
                        iter.inspect(move |item| {
                            let state = context.state_ref(state_handle);
                            let mut state = state.borrow_mut();
                            #root::lattices::Merge::merge(&mut *state, ::std::clone::Clone::clone(item));
                        })
                    }
                    check_input::<_, _, #lattice_type>(#input, #state_ident, #context)
                };
            }
        } else {
            let output = &outputs[0];
            quote_spanned! {op_span=>
                let #ident = {
                    fn check_output<'a, Item, Push, Lat>(
                        push: Push,
                        state_handle: #root::scheduled::state::StateHandle<::std::cell::RefCell<Lat>>,
                        context: &'a #root::scheduled::context::Context,
                    ) -> impl 'a + #root::Pusherator<Item = Item>
                    where
                        Item: ::std::clone::Clone,
                        Push: #root::pusherator::Pusherator<Item = Item>,
                        Lat: 'static + #root::lattice::Merge<Item>,
                    {
                        #root::pusherator::inspect::Inspect::new(move |item| {
                            let state = context.state_ref(state_handle);
                            let mut state = state.borrow_mut();
                            #root::lattices::Merge::merge(&mut *state, ::std::clone::Clone::clone(item))
                        }, push)
                    }
                    check_output::<_, _, #lattice_type>(#output, #state_ident, #context)
                };
            }
        };
        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    },
};
