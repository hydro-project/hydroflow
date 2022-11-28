use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_1,
};

use quote::{quote, quote_spanned};

/// > 2 input streams of type S and T, 1 output stream of type (S, T)
///
/// Forms the cross-join (Cartesian Product) of the items in the input streams, returning all tupled pairs.
///
/// ```hydroflow
/// // should print all 4 pairs of emotion and animal
/// my_join = cross_join();
/// recv_iter(vec!["happy", "sad"]) -> [0]my_join;
/// recv_iter(vec!["dog", "cat"]) -> [1]my_join;
/// my_join -> for_each(|(v1, v2)| println!("({}, {})", v1, v2));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const CROSS_JOIN: OperatorConstraints = OperatorConstraints {
    name: "cross_join",
    hard_range_inn: &(2..=2),
    soft_range_inn: &(2..=2),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: Some(&(|| vec![quote!(0), quote!(1)])),
    ports_out: None,
    num_args: 0,
    input_delaytype_fn: &|_| None,
    write_fn: &(|wc @ &WriteContextArgs { root, op_span, .. },
                 &WriteIteratorArgs { ident, inputs, .. }| {
        let joindata_ident = wc.make_ident("joindata");
        let write_prologue = quote_spanned! {op_span=>
            let mut #joindata_ident = Default::default();
        };

        let lhs = &inputs[0];
        let rhs = &inputs[1];
        let write_iterator = quote_spanned! {op_span=>
            let #lhs = #lhs.map(|a| ((), a));
            let #rhs = #rhs.map(|b| ((), b));
            let #ident = #root::compiled::pull::SymmetricHashJoin::new(#lhs, #rhs, &mut #joindata_ident);
            let #ident = #ident.map(|((), (a, b))| (a, b));
        };

        OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        }
    }),
};
