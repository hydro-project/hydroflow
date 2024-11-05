use quote::quote_spanned;
use syn::parse_quote;

use super::{
    OperatorCategory, OperatorConstraints, OperatorWriteOutput, WriteContextArgs,
    RANGE_0, RANGE_1,
};

// TODO(mingwei):
pub const LATTICE_BIMORPHISM: OperatorConstraints = OperatorConstraints {
    name: "lattice_bimorphism",
    categories: &[OperatorCategory::MultiIn],
    hard_range_inn: &(2..=2),
    soft_range_inn: &(2..=2),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 3,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    has_singleton_output: false,
    ports_inn: Some(|| super::PortListSpec::Fixed(parse_quote! { 0, 1 })),
    ports_out: None,
    input_delaytype_fn: |_| None,
    write_fn: |&WriteContextArgs {
                   root,
                   context,
                   op_span,
                   is_pull,
                   ident,
                   inputs,
                   arguments,
                   arguments_handles,
                   ..
               },
               _| {
        assert!(is_pull);

        let func = &arguments[0];
        let lhs_state_handle = &arguments_handles[1];
        let rhs_state_handle = &arguments_handles[2];

        let lhs_items = &inputs[0];
        let rhs_items = &inputs[1];

        let write_iterator = quote_spanned! {op_span=>
            let #ident = {
                #[inline(always)]
                fn check_inputs<'a, Func, LhsIter, RhsIter, LhsState, RhsState, Output>(
                    mut func: Func,
                    lhs_iter: LhsIter,
                    rhs_iter: RhsIter,
                    lhs_state_handle: #root::scheduled::state::StateHandle<::std::cell::RefCell<LhsState>>,
                    rhs_state_handle: #root::scheduled::state::StateHandle<::std::cell::RefCell<RhsState>>,
                    context: &'a #root::scheduled::context::Context,
                ) -> Option<Output>
                where
                    Func: 'a
                        + #root::lattices::LatticeBimorphism<LhsState, RhsIter::Item, Output = Output>
                        + #root::lattices::LatticeBimorphism<LhsIter::Item, RhsState, Output = Output>,
                    LhsIter: 'a + ::std::iter::Iterator,
                    RhsIter: 'a + ::std::iter::Iterator,
                    LhsState: 'static + ::std::clone::Clone,
                    RhsState: 'static + ::std::clone::Clone,
                    Output: #root::lattices::Merge<Output>,
                {
                    let lhs_state = context.state_ref(lhs_state_handle);
                    let rhs_state = context.state_ref(rhs_state_handle);

                    let mut output = ::std::option::Option::<Output>::None;
                    for lhs_item in lhs_iter {
                        let out_item = func.call(lhs_item, (*rhs_state.borrow()).clone());
                        match &mut output {
                            Some(accum) => {
                                #root::lattices::Merge::merge(accum, out_item);
                            }
                            None => {
                                output = Some(out_item);
                            }
                        }
                    }
                    for rhs_item in rhs_iter {
                        let out_item = func.call((*lhs_state.borrow()).clone(), rhs_item);
                        match &mut output {
                            Some(accum) => {
                                #root::lattices::Merge::merge(accum, out_item);
                            }
                            None => {
                                output = Some(out_item);
                            }
                        }
                    }
                    output
                }
                check_inputs(
                    #func,
                    #lhs_items,
                    #rhs_items,
                    #lhs_state_handle,
                    #rhs_state_handle,
                    &#context,
                ).into_iter()
            };
        };

        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    },
};
