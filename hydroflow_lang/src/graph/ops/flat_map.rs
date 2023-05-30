use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorInstance, OperatorWriteOutput,
    WriteContextArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > 1 input stream, 1 output stream
///
/// > Arguments: A Rust closure that handles an iterator
///
/// For each item `i` passed in, treat `i` as an iterator and map the closure to that
/// iterator to produce items one by one. The type of the input items must be iterable.
///
/// > Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).
///
/// ```hydroflow
/// // should print out each character of each word on a separate line
/// source_iter(vec!["hello", "world"]) -> flat_map(|x| x.chars())
///     -> for_each(|x| println!("{}", x));
/// ```
pub const FLAT_MAP: OperatorConstraints = OperatorConstraints {
    name: "flat_map",
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
    properties: FlowProperties {
        deterministic: FlowPropertyVal::DependsOnArgs,
        monotonic: FlowPropertyVal::DependsOnArgs,
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
                   op_inst: OperatorInstance { arguments, .. },
                   ..
               },
               _| {
        let write_iterator = if is_pull {
            let input = &inputs[0];
            quote_spanned! {op_span=>
                let #ident = #input.flat_map(#arguments);
            }
        } else {
            let output = &outputs[0];
            quote_spanned! {op_span=>
                let #ident = #root::pusherator::map::Map::new(
                    #arguments,
                    #root::pusherator::flatten::Flatten::new(#output)
                );
            }
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    },
};
