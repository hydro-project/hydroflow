use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_1,
};

use quote::quote_spanned;

/// > 1 input stream, 1 output stream
///
/// > Arguments: A Rust closure
/// For each item passed in, apply the closure to generate an item to emit.
///
/// If you do not want to modify the item stream and instead only want to view
/// each item use the [`inspect`](#inspect) operator instead.
///
/// ```hydroflow
/// recv_iter(vec!["hello", "world"]) -> map(|x| x.to_uppercase())
///     -> for_each(|x| println!("{}", x));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const MAP: OperatorConstraints = OperatorConstraints {
    name: "map",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 1,
    input_delaytype_fn: &|_| None,
    write_fn: &(|&WriteContextArgs { root, op_span, .. },
                 &WriteIteratorArgs {
                     ident,
                     inputs,
                     outputs,
                     arguments,
                     is_pull,
                     ..
                 }| {
        let write_iterator = if is_pull {
            let input = &inputs[0];
            quote_spanned! {op_span=>
                let #ident = #input.map(#arguments);
            }
        } else {
            let output = &outputs[0];
            quote_spanned! {op_span=>
                let #ident = #root::pusherator::map::Map::new(#arguments, #output);
            }
        };
        OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        }
    }),
};
