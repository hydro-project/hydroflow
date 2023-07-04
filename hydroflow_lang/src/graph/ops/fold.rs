use quote::quote_spanned;

use super::{
    DelayType, FlowProperties, FlowPropertyVal, OpInstGenerics, OperatorCategory,
    OperatorConstraints, OperatorInstance, OperatorWriteOutput, Persistence, WriteContextArgs,
    RANGE_0, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};

/// > 1 input stream, 1 output stream
///
/// > Arguments: an initial value, and a closure which itself takes two arguments:
/// an 'accumulator', and an element.
///
/// Akin to Rust's built-in fold operator, except that it takes the accumulator by `&mut` instead of by value. Folds every element into an accumulator by applying a closure,
/// returning the final result.
///
/// > Note: The closure has access to the [`context` object](surface_flows.md#the-context-object).
///
/// `fold` can also be provided with one generic lifetime persistence argument, either
/// `'tick` or `'static`, to specify how data persists. With `'tick`, values will only be collected
/// within the same tick. With `'static`, values will be remembered across ticks and will be
/// aggregated with pairs arriving in later ticks. When not explicitly specified persistence
/// defaults to `'static`.
///
/// ```hydroflow
/// // should print `Reassembled vector [1,2,3,4,5]`
/// source_iter([1,2,3,4,5])
///     -> fold::<'tick>(Vec::new(), |accum: &mut Vec<_>, elem| {
///         accum.push(elem);
///     })
///     -> assert([vec![1, 2, 3, 4, 5]]);
/// ```
pub const FOLD: OperatorConstraints = OperatorConstraints {
    name: "fold",
    categories: &[OperatorCategory::Fold],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 2,
    persistence_args: &(0..=1),
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::DependsOnArgs,
        monotonic: FlowPropertyVal::DependsOnArgs,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| Some(DelayType::Stratum),
    write_fn: |wc @ &WriteContextArgs {
                   context,
                   hydroflow,
                   op_span,
                   ident,
                   inputs,
                   is_pull,
                   op_inst:
                       OperatorInstance {
                           arguments,
                           generics:
                               OpInstGenerics {
                                   persistence_args, ..
                               },
                           ..
                       },
                   ..
               },
               diagnostics| {
        assert!(is_pull);

        let persistence = match persistence_args[..] {
            [] => Persistence::Static,
            [a] => a,
            _ => unreachable!(),
        };

        let input = &inputs[0];
        let init = &arguments[0];
        let func = &arguments[1];
        let folddata_ident = wc.make_ident("folddata");
        let accumulator_ident = wc.make_ident("accumulator");
        let iterator_item_ident = wc.make_ident("iterator_item");

        let (write_prologue, write_iterator, write_iterator_after) = match persistence {
            // TODO(mingwei): Issues if initial value is not copy.
            // TODO(mingwei): Might introduce the initial value multiple times on scheduling.
            Persistence::Tick => (
                Default::default(),
                quote_spanned! {op_span=>
                    let #ident = {
                        let mut #accumulator_ident = #init;

                        for #iterator_item_ident in #input {
                            #[allow(clippy::redundant_closure_call)]
                            (#func)(&mut #accumulator_ident, #iterator_item_ident);
                        }

                        ::std::iter::once(#accumulator_ident)
                    };
                },
                Default::default(),
            ),
            Persistence::Static => (
                quote_spanned! {op_span=>
                    let #folddata_ident = #hydroflow.add_state(
                        ::std::cell::Cell::new(::std::option::Option::Some(#init))
                    );
                },
                quote_spanned! {op_span=>
                    let #ident = {
                        let mut #accumulator_ident = #context.state_ref(#folddata_ident).take().expect("FOLD DATA MISSING");

                        for #iterator_item_ident in #input {
                            #[allow(clippy::redundant_closure_call)]
                            (#func)(&mut #accumulator_ident, #iterator_item_ident);
                        }

                        #context.state_ref(#folddata_ident).set(
                            ::std::option::Option::Some(::std::clone::Clone::clone(&#accumulator_ident))
                        );

                        ::std::iter::once(#accumulator_ident)
                    };
                },
                quote_spanned! {op_span=>
                    #context.schedule_subgraph(#context.current_subgraph(), false);
                },
            ),
            Persistence::Mutable => {
                diagnostics.push(Diagnostic::spanned(
                    op_span,
                    Level::Error,
                    "An implementation of 'mutable does not exist",
                ));
                return Err(());
            }
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
