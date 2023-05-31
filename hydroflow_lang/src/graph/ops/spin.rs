use quote::quote_spanned;

use super::{
    FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints, OperatorWriteOutput,
    WriteContextArgs, RANGE_0, RANGE_1,
};

/// This operator will trigger the start of new ticks in order to repeat, which will cause spinning-like behavior.
pub const SPIN: OperatorConstraints = OperatorConstraints {
    name: "spin",
    categories: &[OperatorCategory::Source],
    hard_range_inn: RANGE_0,
    soft_range_inn: RANGE_0,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Yes,
        monotonic: FlowPropertyVal::No,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |&WriteContextArgs {
                   context,
                   op_span,
                   ident,
                   is_pull,
                   ..
               },
               _| {
        assert!(is_pull);
        let write_iterator = quote_spanned! {op_span=>
            let #ident = ::std::iter::once(());
        };
        let write_iterator_after = quote_spanned! {op_span=>
            #context.schedule_subgraph(#context.current_subgraph(), true);
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            write_iterator_after,
            ..Default::default()
        })
    },
};
