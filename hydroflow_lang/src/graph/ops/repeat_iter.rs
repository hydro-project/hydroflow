use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

#[hydroflow_internalmacro::operator_docgen]
pub const REPEAT_ITER: OperatorConstraints = OperatorConstraints {
    name: "repeat_iter",
    hard_range_inn: RANGE_0,
    soft_range_inn: RANGE_0,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 1,
    input_delaytype_fn: &|_| None,
    write_fn: &(|&WriteContextArgs { op_span, .. },
                 &WriteIteratorArgs {
                     ident, arguments, ..
                 },
                 _| {
        let write_iterator = quote_spanned! {op_span=>
            let mut #ident = {
                #[inline(always)]
                fn check_iter<IntoIter: ::std::iter::IntoIterator<Item = Item>, Item>(into_iter: IntoIter) -> impl ::std::iter::Iterator<Item = Item> {
                    ::std::iter::IntoIterator::into_iter(into_iter)
                }
                check_iter(#arguments)
            };
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    }),
};
