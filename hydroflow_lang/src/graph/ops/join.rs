use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_1,
};

use quote::quote_spanned;
use syn::parse_quote;

/// > 2 input streams of type <(K, V1)> and <(K, V2)>, 1 output stream of type <(K, (V1, V2))>
///
/// Forms the equijoin of the tuples in the input streams by their first (key) attribute. Note that the result nests the 2nd input field (values) into a tuple in the 2nd output field.
///
/// ```hydroflow
/// // should print `(hello, (world, cleveland))`
/// my_join = join();
/// source_iter(vec![("hello", "world"), ("stay", "gold")]) -> [0]my_join;
/// source_iter(vec![("hello", "cleveland")]) -> [1]my_join;
/// my_join -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const JOIN: OperatorConstraints = OperatorConstraints {
    name: "join",
    hard_range_inn: &(2..=2),
    soft_range_inn: &(2..=2),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: Some(&(|| parse_quote! { 0, 1 })),
    ports_out: None,
    num_args: 0,
    input_delaytype_fn: &|_| None,
    write_fn: &(|wc @ &WriteContextArgs { root, op_span, .. },
                 &WriteIteratorArgs { ident, inputs, .. },
                 _| {
        let joindata_ident = wc.make_ident("joindata");
        let write_prologue = quote_spanned! {op_span=>
            let mut #joindata_ident = Default::default();
        };

        let lhs = &inputs[0];
        let rhs = &inputs[1];
        let write_iterator = quote_spanned! {op_span=>
            let #ident = #root::compiled::pull::SymmetricHashJoin::new(#lhs, #rhs, &mut #joindata_ident);
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    }),
};
