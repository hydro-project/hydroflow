use crate::graph::PortIndexValue;

use super::{
    DelayType, OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs,
    RANGE_0, RANGE_1,
};

use quote::{quote_spanned, ToTokens};
use syn::parse_quote;

/// > 2 input streams the first of type (K, T), the second of type K,
/// > with output type (K, T)
///
/// For a given tick, computes the anti-join of the items in the input
/// streams, returning items in the `pos` input that do not have matching keys
/// in the `neg` input.
///
/// ```hydroflow
/// // should print ("elephant", 3)
/// diff = anti_join();
/// source_iter(vec![("dog", 1), ("cat", 2), ("elephant", 3)]) -> [pos]diff;
/// source_iter(vec!["dog", "cat", "gorilla"]) -> [neg]diff;
/// diff -> for_each(|v: (_, _)| println!("{}, {}", v.0, v.1));
/// // elephant 3
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const ANTI_JOIN: OperatorConstraints = OperatorConstraints {
    name: "anti_join",
    hard_range_inn: &(2..=2),
    soft_range_inn: &(2..=2),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: Some(&|| super::PortListSpec::Fixed(parse_quote! { pos, neg })),
    ports_out: None,
    input_delaytype_fn: &|idx| match idx {
        PortIndexValue::Path(path) if "neg" == path.to_token_stream().to_string() => {
            Some(DelayType::Stratum)
        }
        _else => None,
    },
    write_fn: &(|wc @ &WriteContextArgs {
                     root,
                     context,
                     op_span,
                     ..
                 },
                 &WriteIteratorArgs { ident, inputs, .. },
                 _| {
        let handle_ident = wc.make_ident("diffdata_handle");
        let write_prologue = quote_spanned! {op_span=>
            let #handle_ident = df.add_state(std::cell::RefCell::new(
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
                let #ident = #input_pos.filter(move |x| !#negset_ident.contains(&x.0));
            }
        };
        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    }),
};
