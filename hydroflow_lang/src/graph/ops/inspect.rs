use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > Arguments: A single closure `FnMut(&Item)`.
///
/// An operator which allows you to "inspect" each element of a stream without
/// modifying it. The closure is called on a reference to each item. This is
/// mainly useful for debugging as in the example below, and it is generally an
/// anti-pattern to provide a closure with side effects.
///
/// > Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).
///
/// ```hydroflow
/// source_iter([1, 2, 3, 4]) -> inspect(|&x| println!("{}", x)) -> null();
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const INSPECT: OperatorConstraints = OperatorConstraints {
    name: "inspect",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 1,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    input_delaytype_fn: |_| None,
    write_fn: |&WriteContextArgs { root, op_span, .. },
               &WriteIteratorArgs {
                   ident,
                   inputs,
                   outputs,
                   arguments,
                   is_pull,
                   ..
               },
               _| {
        let write_iterator = if is_pull {
            let input = &inputs[0];
            quote_spanned! {op_span=>
                let #ident = #input.inspect(#arguments);
            }
        } else {
            let output = &outputs[0];
            quote_spanned! {op_span=>
                let #ident = #root::pusherator::inspect::Inspect::new(#arguments, #output);
            }
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    },
};
