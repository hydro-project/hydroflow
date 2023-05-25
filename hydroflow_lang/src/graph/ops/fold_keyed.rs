use quote::{quote_spanned, ToTokens};

use super::{
    DelayType, FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorWriteOutput,
    Persistence, WriteContextArgs, RANGE_1,
};
use crate::graph::{OpInstGenerics, OperatorInstance};

/// > 1 input stream of type `(K, V1)`, 1 output stream of type `(K, V2)`.
/// The output will have one tuple for each distinct `K`, with an accumulated value of type `V2`.
///
/// If the input and output value types are the same and do not require initialization then use
/// [`reduce_keyed`](#reduce_keyed).
///
/// > Arguments: two Rust closures. The first generates an initial value per group. The second
/// itself takes two arguments: an 'accumulator', and an element. The second closure returns the
/// value that the accumulator should have for the next iteration.
///
/// A special case of `fold`, in the spirit of SQL's GROUP BY and aggregation constructs. The input
/// is partitioned into groups by the first field, and for each group the values in the second
/// field are accumulated via the closures in the arguments.
///
/// > Note: The closures have access to the [`context` object](surface_flows.md#the-context-object).
///
/// `fold_keyed` can also be provided with one generic lifetime persistence argument, either
/// `'tick` or `'static`, to specify how data persists. With `'tick`, values will only be collected
/// within the same tick. With `'static`, values will be remembered across ticks and will be
/// aggregated with pairs arriving in later ticks. When not explicitly specified persistence
/// defaults to `'static`.
///
/// `fold_keyed` can also be provided with two type arguments, the key type `K` and aggregated
/// output value type `V2`. This is required when using `'static` persistence if the compiler
/// cannot infer the types.
///
/// ```hydroflow
/// source_iter([("toy", 1), ("toy", 2), ("shoe", 11), ("shoe", 35), ("haberdashery", 7)])
///     -> fold_keyed(|| 0, |old: &mut u32, val: u32| *old += val)
///     -> for_each(|(k, v)| println!("Total for group {} is {}", k, v));
/// ```
///
/// Example using `'tick` persistence:
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_stream(input_recv)
///         -> fold_keyed::<'tick, &str, String>(String::new, |old: &mut _, val| {
///             *old += val;
///             *old += ", ";
///         })
///         -> for_each(|(k, v)| println!("({:?}, {:?})", k, v));
/// };
///
/// input_send.send(("hello", "oakland")).unwrap();
/// input_send.send(("hello", "berkeley")).unwrap();
/// input_send.send(("hello", "san francisco")).unwrap();
/// flow.run_available();
/// // ("hello", "oakland, berkeley, san francisco, ")
///
/// input_send.send(("hello", "palo alto")).unwrap();
/// flow.run_available();
/// // ("hello", "palo alto, ")
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const FOLD_KEYED: OperatorConstraints = OperatorConstraints {
    name: "fold_keyed",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 2,
    persistence_args: &(0..=1),
    type_args: &(0..=2),
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
                   hydroflow,
                   context,
                   op_span,
                   ident,
                   inputs,
                   is_pull,
                   root,
                   op_inst:
                       OperatorInstance {
                           arguments,
                           generics:
                               OpInstGenerics {
                                   persistence_args,
                                   type_args,
                                   ..
                               },
                           ..
                       },
                   ..
               },
               _| {
        assert!(is_pull);

        let persistence = match persistence_args[..] {
            [] => Persistence::Static,
            [a] => a,
            _ => unreachable!(),
        };

        let generic_type_args = [
            type_args
                .get(0)
                .map(ToTokens::to_token_stream)
                .unwrap_or(quote_spanned!(op_span=> _)),
            type_args
                .get(1)
                .map(ToTokens::to_token_stream)
                .unwrap_or(quote_spanned!(op_span=> _)),
        ];

        let input = &inputs[0];
        let initfn = &arguments[0];
        let aggfn = &arguments[1];

        let (write_prologue, write_iterator, write_iterator_after) = match persistence {
            Persistence::Tick => {
                let groupbydata_ident = wc.make_ident("groupbydata");
                let hashtable_ident = wc.make_ident("hashtable");

                (
                    quote_spanned! {op_span=>
                        let #groupbydata_ident = #hydroflow.add_state(::std::cell::RefCell::new(#root::rustc_hash::FxHashMap::<#( #generic_type_args ),*>::default()));
                    },
                    quote_spanned! {op_span=>
                        let mut #hashtable_ident = #context.state_ref(#groupbydata_ident).borrow_mut();

                        {
                            #[inline(always)]
                            fn check_input<Iter: ::std::iter::Iterator<Item = (A, B)>, A: ::std::clone::Clone, B: ::std::clone::Clone>(iter: Iter)
                                -> impl ::std::iter::Iterator<Item = (A, B)> { iter }

                            for kv in check_input(#input) {
                                let entry = #hashtable_ident.entry(kv.0).or_insert_with(#initfn);
                                #[allow(clippy::redundant_closure_call)] (#aggfn)(entry, kv.1);
                            }
                        }

                        let #ident = #hashtable_ident.drain();
                    },
                    Default::default(),
                )
            }
            Persistence::Static => {
                let groupbydata_ident = wc.make_ident("groupbydata");
                let hashtable_ident = wc.make_ident("hashtable");

                (
                    quote_spanned! {op_span=>
                        let #groupbydata_ident = #hydroflow.add_state(::std::cell::RefCell::new(#root::rustc_hash::FxHashMap::<#( #generic_type_args ),*>::default()));
                    },
                    quote_spanned! {op_span=>
                        let mut #hashtable_ident = #context.state_ref(#groupbydata_ident).borrow_mut();

                        {
                            #[inline(always)]
                            fn check_input<Iter: ::std::iter::Iterator<Item = (A, B)>, A: ::std::clone::Clone, B: ::std::clone::Clone>(iter: Iter)
                                -> impl ::std::iter::Iterator<Item = (A, B)> { iter }

                            for kv in check_input(#input) {
                                let entry = #hashtable_ident.entry(kv.0).or_insert_with(#initfn);
                                #[allow(clippy::redundant_closure_call)] (#aggfn)(entry, kv.1);
                            }
                        }

                        let #ident = #hashtable_ident
                            .iter()
                            .map(#[allow(suspicious_double_ref_op, clippy::clone_on_copy)] |(k, v)| (k.clone(), v.clone()));
                    },
                    quote_spanned! {op_span=>
                        #context.schedule_subgraph(#context.current_subgraph(), false);
                    },
                )
            }
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
