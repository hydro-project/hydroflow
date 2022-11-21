use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_ANY,
};

use quote::quote_spanned;

/// > unbounded number of input streams of any types, unbounded number of output streams of type `()`
///
/// As a source, generates nothing. As a sink, absorbs anything with no effect.
///
/// ```hydroflow
/// // should print `1, 2, 3, 4, 5, 6, a, b, c` across 9 lines
/// null() -> for_each(|_: ()| panic!());
/// recv_iter([1,2,3]) -> map(|i| println!("{}", i)) -> null();
/// null_src = null();
/// null_sink = null();
/// null_src[0] -> for_each(|_: ()| panic!());
/// // note: use `for_each()` (or `inspect()`) instead of this:
/// recv_iter([4,5,6]) -> map(|i| println!("{}", i)) -> [0]null_sink;
/// ```

#[hydroflow_internalmacro::operator_docgen]
pub const NULL: OperatorConstraints = OperatorConstraints {
    name: "null",
    hard_range_inn: RANGE_ANY,
    soft_range_inn: RANGE_ANY,
    hard_range_out: RANGE_ANY,
    soft_range_out: RANGE_ANY,
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
            quote_spanned! {op_span=>
                (#(#inputs.for_each(std::mem::drop)),*);
                let #ident = std::iter::empty();
            }
        } else {
            quote_spanned! {op_span=>
                #[allow(clippy::let_unit_value)]
                let _ = (#(#outputs),*);
                let #ident = #root::pusherator::for_each::ForEach::new(std::mem::drop);
            }
        };
        OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        }
    }),
};
