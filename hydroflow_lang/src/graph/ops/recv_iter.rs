use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// > 0 input streams, 1 output stream
///
/// > Arguments: An iterable Rust object.
/// Takes the iterable object and delivers its elements downstream
/// one by one.
///
/// Note that all elements are emitted during the first epoch.
///
/// ```hydroflow
///     recv_iter(vec!["Hello", "World"])
///         -> for_each(|x| println!("{}", x));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const RECV_ITER: OperatorConstraints = OperatorConstraints {
    name: "recv_iter",
    hard_range_inn: RANGE_0,
    soft_range_inn: RANGE_0,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 1,
    input_delaytype_fn: &|_| None,
    write_fn: &(|wc @ &WriteContextArgs { op_span, .. },
                 &WriteIteratorArgs {
                     ident, arguments, ..
                 }| {
        let iter_ident = wc.make_ident("iter");
        let write_prologue = quote_spanned! {op_span=>
            let mut #iter_ident = std::iter::IntoIterator::into_iter(#arguments);
        };
        let write_iterator = quote_spanned! {op_span=>
            let #ident = #iter_ident.by_ref();
        };
        OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        }
    }),
};
