use super::{
    DelayType, OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs,
    RANGE_1,
};

use quote::quote_spanned;

/// Takes a stream as input and produces a version of the stream as output
/// sorted according to the key extracted by the closure.
///
/// ```hydroflow
/// // should print (1, 'z'), (2, 'y'), (3, 'x') (in order)
/// source_iter(vec![(2, 'y'), (3, 'x'), (1, 'z')])
///     -> sort_by(|(k, _v)| k)
///     -> for_each(|x| println!("{:?}", x));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const SORTBY: OperatorConstraints = OperatorConstraints {
    name: "sort_by",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 1,
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
        let write_iterator = quote_spanned! {op_span=>
            let mut tmp = #input.collect::<Vec<_>>();
            // TODO: remove the clone as shown in addendum here: https://stackoverflow.com/questions/56105305/how-to-sort-a-vec-of-structs-by-a-string-field
            tmp.sort_unstable_by_key(#arguments.clone());
            let #ident = tmp.into_iter();
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    }),
};
