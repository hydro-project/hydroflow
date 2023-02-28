use crate::graph::PortIndexValue;

use super::{
    DelayType, FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorWriteOutput,
    WriteContextArgs, RANGE_0, RANGE_1,
};

use quote::{quote_spanned, ToTokens};
use syn::parse_quote;

/// > 2 input streams of the same type T, 1 output stream of type T
///
/// For a given tick, forms the set difference of the items in the input
/// streams, returning items in the `pos` input that are not found in the
/// `neg` input.
///
/// ```hydroflow
/// // should print "elephant"
/// source_iter(vec!["dog", "cat", "elephant"]) -> [pos]diff;
/// source_iter(vec!["dog", "cat", "gorilla"]) -> [neg]diff;
/// diff = difference() -> for_each(|v| println!("{}", v));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const DIFFERENCE: OperatorConstraints = OperatorConstraints {
    name: "difference",
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
            let #handle_ident = #hydroflow.add_state(std::cell::RefCell::new(
                #root::lang::monotonic_map::MonotonicMap::<_, std::collections::HashSet<_>>::default(),
            ));
        };
        let write_iterator = {
            let borrow_ident = wc.make_ident("borrow");
            let negset_ident = wc.make_ident("negset");

            let input_neg = &inputs[0]; // N before P
            let input_pos = &inputs[1];
            quote_spanned! {op_span=>
                let mut #borrow_ident = #context.state_ref(#handle_ident).borrow_mut();
                let #negset_ident = #borrow_ident
                    .try_insert_with((#context.current_tick(), #context.current_stratum()), || {
                        #input_neg.collect()
                    });
                let #ident = #input_pos.filter(move |x| !#negset_ident.contains(x));
            }
        };
        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    },
};
