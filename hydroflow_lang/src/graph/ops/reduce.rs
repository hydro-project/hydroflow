use super::{
    DelayType, OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs,
    RANGE_1,
};

use quote::quote_spanned;

#[hydroflow_internalmacro::operator_docgen]
pub const REDUCE: OperatorConstraints = OperatorConstraints {
    name: "reduce",
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
                 }| {
        assert!(is_pull);
        let input = &inputs[0];
        let write_iterator = quote_spanned! {op_span=>
            let #ident = #input.reduce(#arguments).into_iter();
        };
        OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        }
    }),
};
