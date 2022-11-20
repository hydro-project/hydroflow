use super::{
    DelayType, OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs,
    RANGE_1,
};

use quote::quote_spanned;

/// Takes one stream as input and filters out any duplicate occurrences. The output
/// contains all unique values from the input.
///
///
/// ```hydroflow
/// // should print 1, 2, 3 (in any order)
/// recv_iter(vec![1, 1, 2, 3, 2, 1, 3])
///     -> unique()
///     -> for_each(|x| println!("{}", x));
/// ```

#[hydroflow_internalmacro::operator_docgen]
pub const UNIQUE: OperatorConstraints = OperatorConstraints {
    name: "unique",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 0,
    input_delaytype_fn: &|_| Some(DelayType::Stratum),
    write_fn: &(|&WriteContextArgs { op_span, .. },
                 &WriteIteratorArgs {
                     ident,
                     inputs,
                     is_pull,
                     ..
                 }| {
        assert!(is_pull);
        let input = &inputs[0];
        let write_iterator = quote_spanned! {op_span=>
            let #ident = #input.fold(
                std::collections::HashSet::new(),
                |mut prev, nxt| {prev.insert(nxt); prev}
            ).into_iter();
        };
        OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        }
    }),
};
