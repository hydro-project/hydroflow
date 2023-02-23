use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_0,
    RANGE_1, RANGE_ANY,
};

use quote::{quote_spanned, ToTokens};

/// > *n* input streams of the same type, 1 output stream of the same type
///
/// Merges an arbitrary number of input streams into a single stream. Each input sequence is a subsequence of the output, but no guarantee is given on how the inputs are interleaved.
///
/// Since `merge` has multiple input streams, it needs to be assigned to
/// a variable to reference its multiple input ports across statements.
///
/// ```hydroflow
/// source_iter(vec!["hello", "world"]) -> my_merge;
/// source_iter(vec!["stay", "gold"]) -> my_merge;
/// source_iter(vec!["don\'t", "give", "up"]) -> my_merge;
/// my_merge = merge() -> map(|x| x.to_uppercase())
///     -> for_each(|x| println!("{}", x));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const MERGE: OperatorConstraints = OperatorConstraints {
    name: "merge",
    hard_range_inn: RANGE_ANY,
    soft_range_inn: &(2..),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    input_delaytype_fn: |_| None,
    write_fn: |&WriteContextArgs { op_span, .. },
               &WriteIteratorArgs {
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   ..
               },
               _| {
        let write_iterator = if is_pull {
            let chains = inputs
                .iter()
                .map(|i| i.to_token_stream())
                .reduce(|a, b| quote_spanned! {op_span=> check_inputs(#a, #b) })
                .unwrap_or_else(|| quote_spanned! {op_span=> std::iter::empty() });
            quote_spanned! {op_span=>
                let #ident = {
                    #[allow(unused)]
                    #[inline(always)]
                    fn check_inputs<A: ::std::iter::Iterator<Item = Item>, B: ::std::iter::Iterator<Item = Item>, Item>(a: A, b: B) -> impl ::std::iter::Iterator<Item = Item> {
                        a.chain(b)
                    }
                    #chains
                };
            }
        } else {
            assert_eq!(1, outputs.len());
            let output = &outputs[0];
            quote_spanned! {op_span=>
                let #ident = #output;
            }
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    },
};
