use super::{
    DelayType, OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs,
    RANGE_1,
};

use quote::quote_spanned;

/// > 1 input stream, 1 output stream
///
/// > Arguments: an initial value, and a closure which itself takes two arguments:
/// an 'accumulator', and an element. The closure returns the value that the accumulator should have for the next iteration.
///
/// Akin to Rust's built-in fold operator. Folds every element into an accumulator by applying a closure,
/// returning the final result.
///
/// ```hydroflow
/// // should print `Reassembled vector [1,2,3,4,5]`
/// source_iter([1,2,3,4,5])
///     -> fold(Vec::new(), |mut accum, elem| {
///         accum.push(elem);
///         accum
///     })
///     -> for_each(|e| println!("Ressembled vector {:?}", e));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const FOLD: OperatorConstraints = OperatorConstraints {
    name: "fold",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 2,
    input_delaytype_fn: &|_| Some(DelayType::Stratum),
    write_fn: &(|&WriteContextArgs { op_span, .. },
                 &WriteIteratorArgs {
                     ident,
                     inputs,
                     arguments,
                     is_pull,
                     ..
                 },
                 _| {
        assert!(is_pull);
        let input = &inputs[0];
        // TODO(mingwei): Issues if initial value is not copy.
        // TODO(mingwei): Might introduce the initial value multiple times on scheduling.
        let write_iterator = quote_spanned! {op_span=>
            let #ident = std::iter::once(#input.fold(#arguments));
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    }),
};
