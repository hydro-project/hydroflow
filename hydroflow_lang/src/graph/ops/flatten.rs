use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorWriteOutput, WriteContextArgs,
    RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > 1 input stream, 1 output stream
///
/// For each item `i` passed in, treat `i` as an iterator and produce its items one by one.
/// The type of the input items must be iterable.
///
/// ```hydroflow
/// // should print the numbers 1-6 without any nesting
/// source_iter(vec![[1, 2], [3, 4], [5, 6]]) -> flatten()
/// -> for_each(|x| println!("{}", x));
/// ```
pub const FLATTEN: OperatorConstraints = OperatorConstraints {
    name: "flatten",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::Preserve,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |&WriteContextArgs {
                   root,
                   op_span,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   ..
               },
               _| {
        let write_iterator = if is_pull {
            let input = &inputs[0];
            quote_spanned! {op_span=>
                let #ident = #input.flatten();
            }
        } else {
            let output = &outputs[0];
            quote_spanned! {op_span=>
                let #ident = #root::pusherator::flatten::Flatten::new(#output);
            }
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    },
};
