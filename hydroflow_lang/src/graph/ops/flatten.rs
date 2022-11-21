use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_1,
};

use quote::quote_spanned;

/// > 1 input stream, 1 output stream
///
/// For each item `i` passed in, treat `i` as an iterator and produce its items one by one.
/// The type of the input items must be iterable.
///
/// ```hydroflow
/// // should print the numbers 1-6 without any nesting
/// recv_iter(vec![[1, 2], [3, 4], [5, 6]]) -> flatten()
/// -> for_each(|x| println!("{}", x));
/// ```

#[hydroflow_internalmacro::operator_docgen]
pub const FLATTEN: OperatorConstraints = OperatorConstraints {
    name: "flatten",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 0,
    input_delaytype_fn: &|_| None,
    write_fn: &(|&WriteContextArgs { root, op_span, .. },
                 &WriteIteratorArgs {
                     ident,
                     inputs,
                     outputs,
                     is_pull,
                     ..
                 }| {
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
        OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        }
    }),
};
