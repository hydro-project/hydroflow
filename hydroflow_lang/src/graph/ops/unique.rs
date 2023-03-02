use crate::graph::{OpInstGenerics, OperatorInstance};

use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorWriteOutput, Persistence,
    WriteContextArgs, RANGE_0, RANGE_1,
};

use quote::quote_spanned;

/// Takes one stream as input and filters out any duplicate occurrences. The output
/// contains all unique values from the input.
///
/// ```hydroflow
/// // should print 1, 2, 3 (in any order)
/// source_iter(vec![1, 1, 2, 3, 2, 1, 3])
///     -> unique()
///     -> for_each(|x| println!("{}", x));
/// ```
///
/// `unique` can also be provided with one generic lifetime persistence argument, either
/// `'tick` or `'static`, to specify how data persists. The default is `'static`.
/// With `'tick`, uniqueness is only considered within the current tick, so across multiple ticks
/// duplicate values may be emitted.
/// With `'static`, values will be remembered across ticks and no duplicates will ever be emitted.
///
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<usize>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_stream(input_recv)
///         -> unique::<'tick>()
///         -> for_each(|n| println!("{}", n));
/// };
///
/// input_send.send(3).unwrap();
/// input_send.send(3).unwrap();
/// input_send.send(4).unwrap();
/// input_send.send(3).unwrap();
/// flow.run_available();
/// // 3, 4
///
/// input_send.send(3).unwrap();
/// input_send.send(5).unwrap();
/// flow.run_available();
/// // 3, 5
/// // Note: 3 is emitted again.
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const UNIQUE: OperatorConstraints = OperatorConstraints {
    name: "unique",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: &(0..=1),
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::Preserve,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   op_span,
                   context,
                   hydroflow,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   op_inst:
                       OperatorInstance {
                           generics:
                               OpInstGenerics {
                                   persistence_args, ..
                               },
                           ..
                       },
                   ..
               },
               _| {
        let persistence = match persistence_args[..] {
            [] => Persistence::Static,
            [a] => a,
            _ => unreachable!(),
        };

        let input = &inputs[0];
        let output = &outputs[0];

        match persistence {
            Persistence::Tick => {
                let uniquedata_ident = wc.make_ident("uniquedata");
                let write_prologue = quote_spanned! {op_span=>
                    let #uniquedata_ident = #hydroflow.add_state(std::cell::RefCell::new(
                        #root::lang::monotonic_map::MonotonicMap::<_, std::collections::HashSet<_>>::default(),
                    ));
                };

                let write_iterator = if is_pull {
                    quote_spanned! {op_span=>
                        let #ident = {
                            let mut borrow = #context.state_ref(#uniquedata_ident).borrow_mut();
                            let set = borrow.try_insert_with((#context.current_tick(), #context.current_stratum()), || ::std::collections::HashSet::new());
                            #input.filter(move |x| set.insert(::std::clone::Clone::clone(x)))
                                .collect::<::std::vec::Vec<_>>()
                                .into_iter()
                        };
                    }
                } else {
                    quote_spanned! {op_span=>
                        let #ident = {
                            let mut borrow = #context.state_ref(#uniquedata_ident).borrow_mut();
                            let set = borrow.try_insert_with((#context.current_tick(), #context.current_stratum()), || ::std::collections::HashSet::new());
                            #root::pusherator::unique::Unique::new(set, #output)
                        };
                    }
                };

                Ok(OperatorWriteOutput {
                    write_prologue,
                    write_iterator,
                    ..Default::default()
                })
            }
            Persistence::Static => {
                let uniquedata_ident = wc.make_ident("uniquedata");

                let write_prologue = quote_spanned! {op_span=>
                    let #uniquedata_ident = #hydroflow.add_state(::std::cell::RefCell::new(::std::collections::HashSet::new()));
                };
                let write_iterator = if is_pull {
                    quote_spanned! {op_span=>
                        // TODO(mingwei): Better data structure for this?
                        let #ident = {
                            let mut set = #context.state_ref(#uniquedata_ident).borrow_mut();
                            #input.filter(|x| set.insert(::std::clone::Clone::clone(x)))
                                .collect::<::std::vec::Vec<_>>()
                                .into_iter()
                        };
                    }
                } else {
                    quote_spanned! {op_span=>
                        let #ident = {
                            let set = #context.state_ref(#uniquedata_ident).borrow_mut();
                            #root::pusherator::unique::Unique::new(set, #output)
                        };
                    }
                };

                Ok(OperatorWriteOutput {
                    write_prologue,
                    write_iterator,
                    ..Default::default()
                })
            }
        }
    },
};
