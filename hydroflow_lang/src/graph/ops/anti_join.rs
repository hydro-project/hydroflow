use quote::{quote_spanned, ToTokens};
use syn::parse_quote;

use super::{
    DelayType, FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints,
    OperatorWriteOutput, WriteContextArgs, RANGE_0, RANGE_1,
};
use crate::graph::PortIndexValue;

/// > 2 input streams the first of type (K, T), the second of type K,
/// > with output type (K, T)
///
/// For a given tick, computes the anti-join of the items in the input
/// streams, returning items in the `pos` input that do not have matching keys
/// in the `neg` input.
///
/// ```hydroflow
/// source_iter(vec![("dog", 1), ("cat", 2), ("elephant", 3)]) -> [pos]diff;
/// source_iter(vec!["dog", "cat", "gorilla"]) -> [neg]diff;
/// diff = anti_join() -> assert_eq([("elephant", 3)]);
/// ```
pub const ANTI_JOIN: OperatorConstraints = OperatorConstraints {
    name: "anti_join",
    categories: &[OperatorCategory::MultiIn],
    hard_range_inn: &(2..=2),
    soft_range_inn: &(2..=2),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: RANGE_0,
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
                   ..
               },
               _| {
        let handle_ident = wc.make_ident("diffdata_handle");
        let write_prologue = quote_spanned! {op_span=>
            let #handle_ident = #hydroflow.add_state(::std::cell::RefCell::new(
                #root::util::monotonic_map::MonotonicMap::<_, #root::rustc_hash::FxHashSet<_>>::default(),
            ));
        };
        let write_iterator = {
            let borrow_ident = wc.make_ident("borrow");

            let input_neg = &inputs[0]; // N before P
            let input_pos = &inputs[1];
            quote_spanned! {op_span=>
                let mut #borrow_ident = #context.state_ref(#handle_ident).borrow_mut();
                let #ident = {
                    /// Limit error propagation by bounding locally, erasing output iterator type.
                    #[inline(always)]
                    fn check_inputs<'a, K, I1, V, I2>(
                        input_pos: I1,
                        input_neg: I2,
                        borrow_state: &'a mut #root::rustc_hash::FxHashSet<K>,
                    ) -> impl 'a + Iterator<Item = (K, V)>
                    where
                        K: Eq + ::std::hash::Hash + Clone,
                        V: Eq + Clone,
                        I1: 'a + Iterator<Item = (K, V)>,
                        I2: 'a + Iterator<Item = K>,
                    {
                        borrow_state.extend(input_neg);
                        input_pos.filter(move |x| !borrow_state.contains(&x.0))
                    }

                    check_inputs(
                        #input_pos,
                        #input_neg,
                        #borrow_ident.get_mut_clear((#context.current_tick(), #context.current_stratum()))
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
