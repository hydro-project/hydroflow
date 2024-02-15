use quote::{quote_spanned, ToTokens};
use syn::parse_quote;

use super::{
    OperatorCategory, OperatorConstraints, OperatorWriteOutput, WriteContextArgs,
    JOIN_CROSS_JOIN_FLOW_PROP_FN, RANGE_0, RANGE_1,
};
use crate::graph::{GraphEdgeType, OpInstGenerics, OperatorInstance};

// TODO(mingwei):
pub const STATE_JOIN: OperatorConstraints = OperatorConstraints {
    name: "state_join",
    categories: &[OperatorCategory::MultiIn],
    hard_range_inn: &(4..=4),
    soft_range_inn: &(4..=4),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: &(0..=1),
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
                   ident,
                   inputs,
                   op_inst:
                       OperatorInstance {
                           generics: OpInstGenerics { type_args, .. },
                           ..
                       },
                   ..
               },
               _| {
        let join_type =
            type_args
                .first()
                .map(ToTokens::to_token_stream)
                .unwrap_or(quote_spanned!(op_span=>
                    #root::compiled::pull::HalfSetJoinState
                ));

        // TODO: This is really bad.
        // This will break if the user aliases HalfSetJoinState to something else. Temporary hacky solution.
        // Note that cross_join() depends on the implementation here as well.
        let additional_trait_bounds = if join_type.to_string().contains("HalfSetJoinState") {
            quote_spanned!(op_span=>
                + ::std::cmp::Eq
            )
        } else {
            quote_spanned!(op_span=>)
        };

        // let mut make_joindata = |persistence, side| {
        //     let joindata_ident = wc.make_ident(format!("joindata_{}", side));
        //     let borrow_ident = wc.make_ident(format!("joindata_{}_borrow", side));
        //     let (init, borrow) = match persistence {
        //         Persistence::Tick => (
        //             quote_spanned! {op_span=>
        //                 #root::util::monotonic_map::MonotonicMap::new_init(
        //                     #join_type::default()
        //                 )
        //             },
        //             quote_spanned! {op_span=>
        //                 &mut *#borrow_ident.get_mut_clear(#context.current_tick())
        //             },
        //         ),
        //         Persistence::Static => (
        //             quote_spanned! {op_span=>
        //                 #join_type::default()
        //             },
        //             quote_spanned! {op_span=>
        //                 &mut *#borrow_ident
        //             },
        //         ),
        //         Persistence::Mutable => {
        //             diagnostics.push(Diagnostic::spanned(
        //                 op_span,
        //                 Level::Error,
        //                 "An implementation of 'mutable does not exist",
        //             ));
        //             return Err(());
        //         }
        //     };
        //     Ok((joindata_ident, borrow_ident, init, borrow))
        // };

        // let (lhs_joindata_ident, lhs_borrow_ident, lhs_init, lhs_borrow) =
        //     make_joindata(persistences[0], "lhs")?;
        // let (rhs_joindata_ident, rhs_borrow_ident, rhs_init, rhs_borrow) =
        //     make_joindata(persistences[1], "rhs")?;

        // let write_prologue = quote_spanned! {op_span=>
        //     let #lhs_joindata_ident = #hydroflow.add_state(std::cell::RefCell::new(
        //         #lhs_init
        //     ));
        //     let #rhs_joindata_ident = #hydroflow.add_state(std::cell::RefCell::new(
        //         #rhs_init
        //     ));
        // };

        let lhs_items = &inputs[0];
        let rhs_items = &inputs[1];
        let lhs_state = &inputs[2];
        let rhs_state = &inputs[3];

        let lhs_borrow_ident = wc.make_ident("joindata_lhs_borrow");
        let rhs_borrow_ident = wc.make_ident("joindata_rhs_borrow");

        let write_iterator = quote_spanned! {op_span=>
            let mut #lhs_borrow_ident = #context.state_ref(#lhs_state).borrow_mut();
            let mut #rhs_borrow_ident = #context.state_ref(#rhs_state).borrow_mut();
            let #ident = {
                // Limit error propagation by bounding locally, erasing output iterator type.
                #[inline(always)]
                fn check_inputs<'a, K, I1, V1, I2, V2>(
                    lhs: I1,
                    rhs: I2,
                    lhs_state: &'a mut #join_type<K, V1, V2>,
                    rhs_state: &'a mut #join_type<K, V2, V1>,
                    is_new_tick: bool,
                ) -> impl 'a + Iterator<Item = (K, (V1, V2))>
                where
                    K: Eq + std::hash::Hash + Clone,
                    V1: Clone #additional_trait_bounds,
                    V2: Clone #additional_trait_bounds,
                    I1: 'a + Iterator<Item = (K, V1)>,
                    I2: 'a + Iterator<Item = (K, V2)>,
                {
                    #root::compiled::pull::symmetric_hash_join_into_iter(lhs, rhs, lhs_state, rhs_state, is_new_tick)
                }

                check_inputs(
                    #lhs_items,
                    #rhs_items,
                    &mut *#lhs_borrow_ident,
                    &mut *#rhs_borrow_ident,
                    #context.is_first_run_this_tick(),
                )
            };
        };

        // let write_iterator_after =
        // if persistences[0] == Persistence::Static || persistences[1] == Persistence::Static {
        //     quote_spanned! {op_span=>
        //         // TODO: Probably only need to schedule if #*_borrow.len() > 0?
        //         #context.schedule_subgraph(#context.current_subgraph(), false);
        //     }
        // } else {
        //     quote_spanned! {op_span=>}
        // };

        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    },
};
