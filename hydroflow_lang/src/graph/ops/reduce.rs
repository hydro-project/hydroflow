use quote::quote_spanned;

use super::{
    DelayType, OpInstGenerics, OperatorCategory, OperatorConstraints,
    OperatorInstance, OperatorWriteOutput, Persistence, WriteContextArgs, RANGE_0, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};

/// > 1 input stream, 1 output stream
///
/// > Arguments: a closure which itself takes two arguments:
/// > an `&mut Accum` accumulator mutable reference, and an `Item`. The closure should merge the item
/// > into the accumulator.
///
/// Akin to Rust's built-in [`reduce`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.reduce)
/// operator, except that it takes the accumulator by `&mut` instead of by value. Reduces every
/// item into an accumulator by applying a closure, returning the final result.
///
/// > Note: The closure has access to the [`context` object](surface_flows.mdx#the-context-object).
///
/// `reduce` can also be provided with one generic lifetime persistence argument, either
/// `'tick` or `'static`, to specify how data persists. With `'tick`, values will only be collected
/// within the same tick. With `'static`, the accumulated value will be remembered across ticks and
/// items are aggregated with items arriving in later ticks. When not explicitly specified
/// persistence defaults to `'tick`.
///
/// ```hydroflow
/// source_iter([1,2,3,4,5])
///     -> reduce::<'tick>(|accum: &mut _, elem| {
///         *accum *= elem;
///     })
///     -> assert_eq([120]);
/// ```
pub const REDUCE: OperatorConstraints = OperatorConstraints {
    name: "reduce",
    categories: &[OperatorCategory::Fold],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: &(0..=1),
    soft_range_out: &(0..=1),
    num_args: 1,
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
                   inputs,
                   is_pull,
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

        let func = &arguments[0];
        let accumulator_ident = wc.make_ident("accumulator");
        let iterator_item_ident = wc.make_ident("iterator_item");

        let iterator_foreach = quote_spanned! {op_span=>
            #[inline(always)]
            fn call_comb_type<Item>(
                accum: &mut Option<Item>,
                item: Item,
                func: impl Fn(&mut Item, Item),
            ) {
                match accum {
                    accum @ None => *accum = Some(item),
                    Some(accum) => (func)(accum, item),
                }
            }
            #[allow(clippy::redundant_closure_call)]
            call_comb_type(&mut *#accumulator_ident, #iterator_item_ident, #func);
        };

        let mut write_prologue = quote_spanned! {op_span=>
            #[allow(clippy::redundant_closure_call)]
            let #singleton_output_ident = #hydroflow.add_state(
                ::std::cell::RefCell::new(::std::option::Option::None)
            );
        };
        if Persistence::Tick == persistence {
            write_prologue.extend(quote_spanned! {op_span=>
                // Reset the value to the initializer fn at the end of each tick.
                #hydroflow.set_state_tick_hook(#singleton_output_ident, |rcell| { rcell.take(); });
            });
        }

        let write_iterator = if is_pull {
        let input = &inputs[0];
            quote_spanned! {op_span=>
                let #ident = {
                    let mut #accumulator_ident = #context.state_ref(#singleton_output_ident).borrow_mut();

                    #input.for_each(|#iterator_item_ident| {
                        #iterator_foreach
                    });

                    #[allow(clippy::clone_on_copy)]
                    {
                        ::std::iter::IntoIterator::into_iter(::std::clone::Clone::clone(&*#accumulator_ident))
                    }
                };
            }
        } else {
            // Is only push when used as a singleton, so no need to push to `outputs[0]`.
            quote_spanned! {op_span=>
                let #ident = {
                    #root::pusherator::for_each::ForEach::new(|#iterator_item_ident| {
                        let mut #accumulator_ident = #context.state_ref(#singleton_output_ident).borrow_mut();
                        #iterator_foreach
                    })
                };
            }
        };
        let write_iterator_after = if Persistence::Static == persistence {
            quote_spanned! {op_span=>
                #context.schedule_subgraph(#context.current_subgraph(), false);
            }
        } else {
            Default::default()
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
