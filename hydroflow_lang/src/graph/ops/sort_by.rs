use super::{
    DelayType, OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs,
    RANGE_0, RANGE_1,
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
pub const SORT_BY: OperatorConstraints = OperatorConstraints {
    name: "sort_by",
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
    input_delaytype_fn: &|_| Some(DelayType::Stratum),
    write_fn: &(|&WriteContextArgs { root, op_span, .. },
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
            #root::util::sort_unstable_by_key_hrtb(&mut tmp, #arguments);
            let #ident = tmp.into_iter();
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    }),
};
