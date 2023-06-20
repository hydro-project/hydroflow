use quote::quote_spanned;

use super::{
    FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints, OperatorWriteOutput,
    WriteContextArgs, RANGE_0, RANGE_1,
};
use crate::graph::OperatorInstance;

/// > 1 input stream, 1 optional output stream
/// > Arguments: A Vector, Slice, or Array containing objects that will be compared to the input stream.
///
/// The input stream will be collected into a vector and then compared with the input argument. If there is a difference then the program will panic.
///
/// assert() is mainly useful for testing and documenting the behavior of hydroflow code inline.
///
/// assert() will pass through the items to its output channel unchanged and in-order.
///
/// ```hydroflow
/// source_iter([1, 2, 3])
///     -> assert([1, 2, 3])
///     -> assert([1, 2, 3]);
/// ```
pub const ASSERT: OperatorConstraints = OperatorConstraints {
    name: "assert",
    categories: &[OperatorCategory::Control],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: &(0..=1),
    soft_range_out: &(0..=1),
    num_args: 1,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::DependsOnArgs,
        monotonic: FlowPropertyVal::DependsOnArgs,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |wc @ &WriteContextArgs {
                   context,
                   hydroflow,
                   root,
                   op_span,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   op_inst: OperatorInstance { arguments, .. },
                   ..
               },
               _| {
        let vec = &arguments[0];

        let (write_prologue, write_iterator, write_iterator_after) = if is_pull {
            let input = &inputs[0];

            (
                Default::default(),
                quote_spanned! {op_span=>
                    let input_vec = #input.collect::<::std::vec::Vec<_>>();

                    assert_eq!(input_vec, #vec, "lhs = dataflow input, rhs = expected value (provided as argument to assert())");

                    let #ident = input_vec.into_iter();
                },
                Default::default(),
            )
        } else {
            let assert_data_ident = wc.make_ident("assert_data");

            (
                quote_spanned! {op_span=>
                    let #assert_data_ident = #hydroflow.add_state(
                        ::std::cell::Cell::new(::std::vec::Vec::default())
                    );
                },
                {
                    match outputs {
                        [] => {
                            quote_spanned! {op_span=>
                                let #ident = #root::pusherator::for_each::ForEach::new(|v| {
                                    let mut accum = #context.state_ref(#assert_data_ident).take();
                                    accum.push(v);
                                    #context.state_ref(#assert_data_ident).set(accum);
                                });
                            }
                        }
                        [output] => {
                            quote_spanned! {op_span=>
                                let #ident = #root::pusherator::inspect::Inspect::new(|v| {
                                    let mut accum = #context.state_ref(#assert_data_ident).take();
                                    accum.push(v);
                                    #context.state_ref(#assert_data_ident).set(accum);
                                }, #output);
                            }
                        }
                        _ => panic!(),
                    }
                },
                quote_spanned! {op_span=>
                    let accum = #context.state_ref(#assert_data_ident).take();

                    assert_eq!(accum, #vec, "lhs = dataflow input, rhs = expected value (provided as argument to assert())");

                    #context.state_ref(#assert_data_ident).set(accum);
                },
            )
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
