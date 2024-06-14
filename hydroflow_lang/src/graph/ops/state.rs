use quote::{quote_spanned, ToTokens};

use super::{
    OpInstGenerics, OperatorCategory, OperatorConstraints, OperatorInstance, OperatorWriteOutput,
    Persistence, WriteContextArgs, LATTICE_FOLD_REDUCE_FLOW_PROP_FN, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};

// TODO(mingwei): Improve example when things are more stable.
/// A lattice-based state operator, used for accumulating lattice state
///
/// Emits both a referenceable singleton and (optionally) a pass-through stream. In the future the
/// pass-through stream may be deduplicated.
///
/// ```hydroflow
/// use std::collections::HashSet;
///
/// use lattices::set_union::{CartesianProductBimorphism, SetUnionHashSet, SetUnionSingletonSet};
///
/// my_state = source_iter_delta(0..3)
///     -> map(SetUnionSingletonSet::new_from)
///     -> state::<SetUnionHashSet<usize>>();
/// ```
pub const STATE: OperatorConstraints = OperatorConstraints {
    name: "state",
    categories: &[OperatorCategory::Persistence],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: &(0..=1),
    soft_range_out: &(0..=1),
    num_args: 0,
    persistence_args: &(0..=1),
    type_args: &(0..=1),
    is_external_input: false,
    has_singleton_output: true,
    ports_inn: None,
    ports_out: None,
    input_delaytype_fn: |_| None,
    flow_prop_fn: Some(LATTICE_FOLD_REDUCE_FLOW_PROP_FN),
    write_fn: |&WriteContextArgs {
                   root,
                   context,
                   hydroflow,
                   op_span,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   singleton_output_ident,
                   op_name,
                   op_inst:
                       OperatorInstance {
                           generics:
                               OpInstGenerics {
                                   type_args,
                                   persistence_args,
                                   ..
                               },
                           ..
                       },
                   ..
               },
               diagnostics| {
        let lattice_type = type_args
            .first()
            .map(ToTokens::to_token_stream)
            .unwrap_or(quote_spanned!(op_span=> _));

        let persistence = match persistence_args[..] {
            [] => Persistence::Tick,
            [Persistence::Mutable] => {
                diagnostics.push(Diagnostic::spanned(
                    op_span,
                    Level::Error,
                    format!("{} does not support `'mut`.", op_name),
                ));
                Persistence::Tick
            }
            [a] => a,
            _ => unreachable!(),
        };

        let state_ident = singleton_output_ident;
        let mut write_prologue = quote_spanned! {op_span=>
            let #state_ident = #hydroflow.add_state(::std::cell::RefCell::new(
                <#lattice_type as ::std::default::Default>::default()
            ));
        };
        if Persistence::Tick == persistence {
            write_prologue.extend(quote_spanned! {op_span=>
                #hydroflow.set_state_tick_reset(#state_ident, ::std::default::Default::default);
            });
        }

        // TODO(mingwei): deduplicate codegen
        let write_iterator = if is_pull {
            let input = &inputs[0];
            quote_spanned! {op_span=>
                let #ident = {
                    fn check_input<'a, Item, Iter, Lat>(
                        iter: Iter,
                        state_handle: #root::scheduled::state::StateHandle<::std::cell::RefCell<Lat>>,
                        context: &'a #root::scheduled::context::Context,
                    ) -> impl 'a + ::std::iter::Iterator<Item = Item>
                    where
                        Item: ::std::clone::Clone,
                        Iter: 'a + ::std::iter::Iterator<Item = Item>,
                        Lat: 'static + #root::lattices::Merge<Item>,
                    {
                        iter.inspect(move |item| {
                            let state = context.state_ref(state_handle);
                            let mut state = state.borrow_mut();
                            #root::lattices::Merge::merge(&mut *state, ::std::clone::Clone::clone(item));
                        })
                    }
                    check_input::<_, _, #lattice_type>(#input, #state_ident, #context)
                };
            }
        } else if let Some(output) = outputs.first() {
            quote_spanned! {op_span=>
                let #ident = {
                    fn check_output<'a, Item, Push, Lat>(
                        push: Push,
                        state_handle: #root::scheduled::state::StateHandle<::std::cell::RefCell<Lat>>,
                        context: &'a #root::scheduled::context::Context,
                    ) -> impl 'a + #root::pusherator::Pusherator<Item = Item>
                    where
                        Item: 'a + ::std::clone::Clone,
                        Push: #root::pusherator::Pusherator<Item = Item>,
                        Lat: 'static + #root::lattices::Merge<Item>,
                    {
                        #root::pusherator::inspect::Inspect::new(move |item| {
                            let state = context.state_ref(state_handle);
                            let mut state = state.borrow_mut();
                            #root::lattices::Merge::merge(&mut *state, ::std::clone::Clone::clone(item));
                        }, push)
                    }
                    check_output::<_, _, #lattice_type>(#output, #state_ident, #context)
                };
            }
        } else {
            quote_spanned! {op_span=>
                let #ident = {
                    fn check_output<'a, Item, Lat>(
                        state_handle: #root::scheduled::state::StateHandle<::std::cell::RefCell<Lat>>,
                        context: &'a #root::scheduled::context::Context,
                    ) -> impl 'a + #root::pusherator::Pusherator<Item = Item>
                    where
                        Item: 'a,
                        Lat: 'static + #root::lattices::Merge<Item>,
                    {
                        #root::pusherator::for_each::ForEach::new(move |item| {
                            let state = context.state_ref(state_handle);
                            let mut state = state.borrow_mut();
                            #root::lattices::Merge::merge(&mut *state, item);
                        })
                    }
                    check_output::<_, #lattice_type>(#state_ident, #context)
                };
            }
        };
        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    },
};
