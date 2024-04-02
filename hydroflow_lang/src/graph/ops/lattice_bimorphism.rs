use quote::quote_spanned;
use syn::parse_quote;

use super::{
    GraphEdgeType, OperatorCategory, OperatorConstraints, OperatorWriteOutput, WriteContextArgs,
    JOIN_CROSS_JOIN_FLOW_PROP_FN, RANGE_0, RANGE_1,
};

// TODO(mingwei):
pub const LATTICE_BIMORPHISM: OperatorConstraints = OperatorConstraints {
    name: "lattice_bimorphism",
    categories: &[OperatorCategory::MultiIn],
    hard_range_inn: &(4..=4),
    soft_range_inn: &(4..=4),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 1,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: Some(|| {
        super::PortListSpec::Fixed(parse_quote! { items_0, items_1, state_0, state_1 })
    }),
    ports_out: None,
    input_delaytype_fn: |_| None,
    input_edgetype_fn: |port| match &*port.to_string() {
        "items_0" => Some(GraphEdgeType::Value),
        "items_1" => Some(GraphEdgeType::Value),
        "state_0" => Some(GraphEdgeType::Reference),
        "state_1" => Some(GraphEdgeType::Reference),
        _ => None,
    },
    output_edgetype_fn: |_| GraphEdgeType::Value,
    flow_prop_fn: Some(JOIN_CROSS_JOIN_FLOW_PROP_FN),
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   context,
                   op_span,
                   is_pull,
                   ident,
                   inputs,
                   arguments,
                   ..
               },
               _| {
        assert!(is_pull);

        let func = wc.wrap_check_func_arg(&arguments[0]);

        let lhs_items = &inputs[0];
        let rhs_items = &inputs[1];
        let lhs_state = &inputs[2];
        let rhs_state = &inputs[3];

        let write_iterator = quote_spanned! {op_span=>
            let #ident = {
                #[inline(always)]
                fn check_inputs<'a, Func, LhsIter, RhsIter, LhsState, RhsState, Output>(
                    mut func: Func,
                    mut lhs_iter: LhsIter,
                    mut rhs_iter: RhsIter,
                    lhs_state_handle: #root::scheduled::state::StateHandle<::std::cell::RefCell<LhsState>>,
                    rhs_state_handle: #root::scheduled::state::StateHandle<::std::cell::RefCell<RhsState>>,
                    context: &'a #root::scheduled::context::Context,
                ) -> impl 'a + ::std::iter::Iterator<Item = Output>
                where
                    Func: 'a
                        + #root::lattices::LatticeBimorphism<LhsState, RhsIter::Item, Output = Output>
                        + #root::lattices::LatticeBimorphism<LhsIter::Item, RhsState, Output = Output>,
                    LhsIter: 'a + ::std::iter::Iterator,
                    RhsIter: 'a + ::std::iter::Iterator,
                    LhsState: 'static + ::std::clone::Clone,
                    RhsState: 'static + ::std::clone::Clone,
                {
                    let lhs_state = context.state_ref(lhs_state_handle);
                    let rhs_state = context.state_ref(rhs_state_handle);

                    ::std::iter::from_fn(move || {
                        // Use `from_fn` instead of `chain` to dodge multiple ownership of `func`.
                        if let Some(lhs_item) = lhs_iter.next() {
                            Some(func.call(lhs_item, (*rhs_state.borrow()).clone()))
                        } else {
                            let rhs_item = rhs_iter.next()?;
                            Some(func.call((*lhs_state.borrow()).clone(), rhs_item))
                        }
                    })
                }
                check_inputs(
                    #func,
                    #lhs_items,
                    #rhs_items,
                    #lhs_state,
                    #rhs_state,
                    &#context,
                )
            };
        };

        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    },
};
