// TODO(mingwei): deduplicate this with `repeat_iter` somehow.

use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorInstance, OperatorWriteOutput,
    WriteContextArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// Same as [`repeat_iter`](#repeat_iter) but treats the iter as external, meaning this operator
/// will trigger the start of new ticks in order to repeat, causing spinning-like behavior.
#[hydroflow_internalmacro::operator_docgen]
pub const REPEAT_ITER_EXTERNAL: OperatorConstraints = OperatorConstraints {
    name: "repeat_iter_external",
    hard_range_inn: RANGE_0,
    soft_range_inn: RANGE_0,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 1,
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
                   op_inst: OperatorInstance { arguments, .. },
                   ..
               },
               _| {
        let write_iterator = quote_spanned! {op_span=>
            let #ident = {
                #[inline(always)]
                fn check_iter<IntoIter: ::std::iter::IntoIterator<Item = Item>, Item>(into_iter: IntoIter) -> impl ::std::iter::Iterator<Item = Item> {
                    ::std::iter::IntoIterator::into_iter(into_iter)
                }
                check_iter(#arguments)
            };
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
