use quote::quote_spanned;

use super::{
    DelayType, OpInstGenerics, OperatorCategory, OperatorConstraints, OperatorInstance,
    OperatorWriteOutput, Persistence, WriteContextArgs, RANGE_0, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};

/// > 1 input stream, 1 output stream
///
/// > Arguments: two arguments, both closures. The first closure is used to create the initial value for the accumulator, and the second is used to combine new values with the existing accumulator.
/// The second closure takes two two arguments: an 'accumulator', and an element.
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
/// defaults to `'tick`.
///
/// ```hydroflow
/// // should print `Reassembled vector [1,2,3,4,5]`
/// source_iter([1,2,3,4,5])
///     -> fold::<'tick>(Vec::new, |accum: &mut Vec<_>, elem| {
///         accum.push(elem);
///     })
///     -> assert_eq([vec![1, 2, 3, 4, 5]]);
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
    input_delaytype_fn: |_| Some(DelayType::Stratum),
    flow_prop_fn: None,
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
            [] => Persistence::Tick,
            [a] => a,
            _ => unreachable!(),
        };

        let input = &inputs[0];
        let init = &arguments[0];
        let func = &arguments[1];
        let initializer_func_ident = wc.make_ident("initializer_func");
        let folddata_ident = wc.make_ident("folddata");
        let accumulator_ident = wc.make_ident("accumulator");
        let iterator_item_ident = wc.make_ident("iterator_item");

        let (write_prologue, write_iterator, write_iterator_after) = match persistence {
            Persistence::Tick => (
                quote_spanned! {op_span=>
                    let #initializer_func_ident = #init;
                },
                quote_spanned! {op_span=>
                    let #ident = {
                        #[allow(clippy::redundant_closure_call)]
                        let mut #accumulator_ident = (#initializer_func_ident)();

                        #[inline(always)]
                        /// A: accumulator type
                        /// T: iterator item type
                        /// O: output type
                        fn call_comb_type<A, T, O>(a: &mut A, t: T, f: impl Fn(&mut A, T) -> O) -> O {
                            f(a, t)
                        }

                        for #iterator_item_ident in #input {
                            #[allow(clippy::redundant_closure_call)]
                            call_comb_type(&mut #accumulator_ident, #iterator_item_ident, #func);
                        }

                        ::std::iter::once(#accumulator_ident)
                    };
                },
                Default::default(),
            ),
            Persistence::Static => (
                quote_spanned! {op_span=>
                    let #initializer_func_ident = #init;

                    #[allow(clippy::redundant_closure_call)]
                    let #folddata_ident = #hydroflow.add_state(
                        ::std::cell::Cell::new(::std::option::Option::Some((#initializer_func_ident)()))
                    );
                },
                quote_spanned! {op_span=>
                    let #ident = {
                        let mut #accumulator_ident = #context.state_ref(#folddata_ident).take().expect("FOLD DATA MISSING");

                        #[inline(always)]
                        /// A: accumulator type
                        /// T: iterator item type
                        /// O: output type
                        fn call_comb_type<A, T, O>(a: &mut A, t: T, f: impl Fn(&mut A, T) -> O) -> O {
                            f(a, t)
                        }

                        for #iterator_item_ident in #input {
                            #[allow(clippy::redundant_closure_call)]
                            call_comb_type(&mut #accumulator_ident, #iterator_item_ident, #func);
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
