use quote::quote_spanned;

use super::{
    OperatorCategory, OperatorConstraints, OperatorWriteOutput, WriteContextArgs,
    RANGE_0, RANGE_1,
};

/// > 1 input stream, 1 output stream
///
/// > Arguments: A Rust closure that handles an iterator
///
/// For each item `i` passed in, treat `i` as an iterator and map the closure to that
/// iterator to produce items one by one. The type of the input items must be iterable.
///
/// > Note: The closure has access to the [`context` object](surface_flows.mdx#the-context-object).
///
/// ```hydroflow
/// // should print out each character of each word on a separate line
/// source_iter(vec!["hello", "world"])
///     -> flat_map(|x| x.chars())
///     -> assert_eq(['h', 'e', 'l', 'l', 'o', 'w', 'o', 'r', 'l', 'd']);
/// ```
pub const FLAT_MAP: OperatorConstraints = OperatorConstraints {
    name: "flat_map",
    categories: &[OperatorCategory::Flatten],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 1,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    has_singleton_output: false,
    flo_type: None,
    ports_inn: None,
    ports_out: None,
    input_delaytype_fn: |_| None,
    write_fn: |&WriteContextArgs {
                   root,
                   op_span,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   arguments,
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
