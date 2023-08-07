use quote::{quote_spanned, ToTokens};
use syn::parse_quote;

use super::{
    DelayType, FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints,
    OperatorWriteOutput, WriteContextArgs, RANGE_0, RANGE_1, Persistence,
};
use crate::graph::{PortIndexValue, OperatorInstance, OpInstGenerics};
use crate::diagnostic::{Diagnostic, Level};

/// > 2 input streams the first of type (K, T), the second of type K,
/// > with output type (K, T)
///
/// For a given tick, computes the anti-join of the items in the input
/// streams, returning items in the `pos` input --that do not have matching keys
/// in the `neg` input. NOTE this uses multiset semantics on the positive side,
/// so duplicated positive inputs will appear in the output either 0 times (if matched in `neg`)
/// or as many times as they appear in the input (if not matched in `neg`)
///
/// ```hydroflow
/// source_iter(vec![("dog", 1), ("cat", 2), ("elephant", 3)]) -> [pos]diff;
/// source_iter(vec!["dog", "cat", "gorilla"]) -> [neg]diff;
/// diff = anti_join() -> assert_eq([("elephant", 3)]);
/// ```

// This implementation is largely redundant to ANTI_JOIN and should be DRY'ed
pub const ANTI_JOIN_MULTISET: OperatorConstraints = OperatorConstraints {
    name: "anti_join_multiset",
    categories: &[OperatorCategory::MultiIn],
    hard_range_inn: &(2..=2),
    soft_range_inn: &(2..=2),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: &(0..=2),
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: Some(|| super::PortListSpec::Fixed(parse_quote! { pos, neg })),
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::No,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |idx| match idx {
        PortIndexValue::Path(path) if "neg" == path.to_token_stream().to_string() => {
            Some(DelayType::Stratum)
        }
        _else => None,
    },
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   context,
                   hydroflow,
                   op_span,
                   ident,
                   inputs,
                   op_inst:
                       OperatorInstance {
                           generics:
                               OpInstGenerics {
                                   persistence_args,
                                   ..
                               },
                           ..
                       },
                   ..
               },
               diagnostics| {

        let persistences = match persistence_args[..] {
            [] => [Persistence::Tick, Persistence::Tick],
            [a] => [a, a],
            [a, b] => [a, b],
            _ => unreachable!(),
        };

        let mut make_antijoindata = |persistence, side| {
            let antijoindata_ident = wc.make_ident(format!("antijoindata_{}", side));
            let borrow_ident = wc.make_ident(format!("antijoindata_{}_borrow", side));
            let (init, borrow) = match persistence {
                Persistence::Tick => (
                    quote_spanned! {op_span=>
                        #root::util::monotonic_map::MonotonicMap::<_, #root::rustc_hash::FxHashSet<_>>::default()
                    },
                    quote_spanned! {op_span=>
                        &mut *#borrow_ident.get_mut_clear(#context.current_tick())
                    },
                ),
                Persistence::Static => (
                    quote_spanned! {op_span=>
                        #root::rustc_hash::FxHashSet::default()
                    },
                    quote_spanned! {op_span=>
                        &mut *#borrow_ident
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
            Ok((antijoindata_ident, borrow_ident, init, borrow))
        };

        let (neg_antijoindata_ident, neg_borrow_ident, neg_init, neg_borrow) =
                make_antijoindata(persistences[1], "neg")?;

        let write_prologue = quote_spanned! {op_span=>
            let #neg_antijoindata_ident = #hydroflow.add_state(std::cell::RefCell::new(
                #neg_init
            ));
        };

        let input_neg = &inputs[0]; // N before P
        let input_pos = &inputs[1];
        let write_iterator = {
            quote_spanned! {op_span=>
                let mut #neg_borrow_ident = #context.state_ref(#neg_antijoindata_ident).borrow_mut();
                let #ident = {
                    /// Limit error propagation by bounding locally, erasing output iterator type.
                    #[inline(always)]
                    fn check_inputs<'a, K, I1, V, I2>(
                        input_neg: I1,
                        input_pos: I2,
                        neg_state: &'a mut #root::rustc_hash::FxHashSet<K>,
                    ) -> impl 'a + Iterator<Item = (K, V)>
                    where
                        K: Eq + ::std::hash::Hash + Clone,
                        V: Eq + ::std::hash::Hash + Clone,
                        I1: 'a + Iterator<Item = K>,
                        I2: 'a + Iterator<Item = (K, V)>,
                    {
                        neg_state.extend(input_neg);
                        input_pos.filter_map(move |x| {
                            if !neg_state.contains(&x.0){
                                    Some(x)
                            }
                            else {
                                None
                            }
                        })
                    }

                    check_inputs(
                        #input_neg,
                        #input_pos,
                        #neg_borrow,
                    )
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
