use quote::quote_spanned;

use super::{
    DelayType, OpInstGenerics, OperatorCategory, OperatorConstraints, OperatorInstance,
    OperatorWriteOutput, Persistence, WriteContextArgs, RANGE_0, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};

/// > 1 input stream, 1 output stream
///
/// > Arguments: two arguments, both closures. The first closure is used to create the initial
/// > value for the accumulator, and the second is used to combine new items with the existing
/// > accumulator value. The second closure takes two two arguments: an `&mut Accum` accumulated
/// > value, and an `Item`.
///
/// Akin to Rust's built-in [`fold`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.fold)
/// operator, except that it takes the accumulator by `&mut` instead of by value. Folds every item
/// into an accumulator by applying a closure, returning the final result.
///
/// > Note: The closures have access to the [`context` object](surface_flows.mdx#the-context-object).
///
/// `fold` can also be provided with one generic lifetime persistence argument, either
/// `'tick` or `'static`, to specify how data persists. With `'tick`, Items will only be collected
/// within the same tick. With `'static`, the accumulated value will be remembered across ticks and
/// will be aggregated with items arriving in later ticks. When not explicitly specified
/// persistence defaults to `'tick`.
///
/// ```dfir
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
    hard_range_out: &(0..=1),
    soft_range_out: &(0..=1),
    num_args: 2,
    persistence_args: &(0..=1),
    type_args: RANGE_0,
    is_external_input: false,
    has_singleton_output: true,
    flo_type: None,
    ports_inn: None,
    ports_out: None,
    input_delaytype_fn: |_| Some(DelayType::Stratum),
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   context,
                   hydroflow,
                   op_span,
                   ident,
                   is_pull,
                   inputs,
                   singleton_output_ident,
                   op_inst:
                       OperatorInstance {
                           generics:
                               OpInstGenerics {
                                   persistence_args, ..
                               },
                           ..
                       },
                   arguments,
                   ..
               },
               diagnostics| {
        let persistence = match persistence_args[..] {
            [] => Persistence::Tick,
            [a] => a,
            _ => unreachable!(),
        };
        if Persistence::Mutable == persistence {
            diagnostics.push(Diagnostic::spanned(
                op_span,
                Level::Error,
                "An implementation of 'mutable does not exist",
            ));
            return Err(());
        }

        let input = &inputs[0];
        let init = &arguments[0];
        let func = &arguments[1];
        let initializer_func_ident = wc.make_ident("initializer_func");
        let accumulator_ident = wc.make_ident("accumulator");
        let iterator_item_ident = wc.make_ident("iterator_item");

        let iterator_foreach = quote_spanned! {op_span=>
            #[inline(always)]
            fn call_comb_type<Accum, Item>(
                accum: &mut Accum,
                item: Item,
                func: impl Fn(&mut Accum, Item),
            ) {
                (func)(accum, item);
            }
            #[allow(clippy::redundant_closure_call)]
            call_comb_type(&mut *#accumulator_ident, #iterator_item_ident, #func);
        };

        let mut write_prologue = quote_spanned! {op_span=>
            #[allow(unused_mut)]
            let mut #initializer_func_ident = #init;

            #[allow(clippy::redundant_closure_call)]
            let #singleton_output_ident = #hydroflow.add_state(
                ::std::cell::RefCell::new((#initializer_func_ident)())
            );
        };
        if Persistence::Tick == persistence {
            write_prologue.extend(quote_spanned! {op_span=>
                // Reset the value to the initializer fn if it is a new tick.
                #hydroflow.set_state_tick_hook(#singleton_output_ident, move |rcell| { rcell.replace((#initializer_func_ident)()); });
            });
        }
        let write_iterator = if is_pull {
            quote_spanned! {op_span=>
                let #ident = {
                    let mut #accumulator_ident = #context.state_ref(#singleton_output_ident).borrow_mut();

                    #input.for_each(|#iterator_item_ident| {
                        #iterator_foreach
                    });

                    #[allow(clippy::clone_on_copy)]
                    {
                        ::std::iter::once(::std::clone::Clone::clone(&*#accumulator_ident))
                    }
                };
            }
        } else {
            quote_spanned! {op_span=>
                let #ident = {
                    #root::pusherator::for_each::ForEach::new(|#iterator_item_ident| {
                        let mut #accumulator_ident = #context.state_ref(#singleton_output_ident).borrow_mut();
                        #iterator_foreach
                    })
                };
            }
        };
        let write_iterator_after = quote_spanned! {op_span=>
            #context.schedule_subgraph(#context.current_subgraph(), false);
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
