use quote::{quote_spanned, ToTokens};
use syn::parse_quote;

use super::{
    DelayType, FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints,
    OperatorWriteOutput, Persistence, WriteContextArgs, RANGE_0, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};
use crate::graph::{OpInstGenerics, OperatorInstance, PortIndexValue};

/// > 2 input streams of the same type T, 1 output stream of type T
///
/// For a given tick, forms the set difference of the items in the input
/// streams, returning items in the `pos` input that are not found in the
/// `neg` input.
///
/// ```hydroflow
/// source_iter(vec!["dog", "cat", "elephant"]) -> [pos]diff;
/// source_iter(vec!["dog", "cat", "gorilla"]) -> [neg]diff;
/// diff = difference() -> assert(["elephant"]);
/// ```
pub const DIFFERENCE: OperatorConstraints = OperatorConstraints {
    name: "difference",
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
                                   persistence_args, ..
                               },
                           ..
                       },
                   ..
               },
               diagnostics| {
        let handle_ident = wc.make_ident("diffdata_handle");

        let persistence = match &persistence_args[..] {
            [] => Persistence::Tick,
            [Persistence::Tick, Persistence::Tick] => Persistence::Tick,
            [Persistence::Tick, Persistence::Static] => Persistence::Static,
            other => {
                diagnostics.push(Diagnostic::spanned(
                    op_span,
                    Level::Error,
                    &*format!(
                        "Unexpected persistence arguments for difference, expected two arguments with the first as `'tick`, got {:?}", // or whatever
                        other
                    ),
                ));

                Persistence::Tick
            }
        };

        let (write_prologue, write_iterator) = {
            let borrow_ident = wc.make_ident("borrow");
            let negset_ident = wc.make_ident("negset");

            let input_neg = &inputs[0]; // N before P
            let input_pos = &inputs[1];

            let (write_prologue, get_set) = match persistence {
                Persistence::Tick => (
                    quote_spanned! {op_span=>
                        let #handle_ident = #hydroflow.add_state(std::cell::RefCell::new(
                            #root::util::monotonic_map::MonotonicMap::<_, #root::rustc_hash::FxHashSet<_>>::default(),
                        ));
                    },
                    quote_spanned! {op_span=>
                        let mut #borrow_ident = #context.state_ref(#handle_ident).borrow_mut();
                        let #negset_ident = #borrow_ident
                            .get_mut_with((#context.current_tick(), #context.current_stratum()), || {
                                #input_neg.collect()
                            });
                    },
                ),

                Persistence::Static => (
                    quote_spanned! {op_span=>
                        let #handle_ident = #hydroflow.add_state(::std::cell::RefCell::new(#root::rustc_hash::FxHashSet::default()));
                    },
                    quote_spanned! {op_span=>
                        let mut #negset_ident = #context.state_ref(#handle_ident).borrow_mut();
                        #negset_ident.extend(#input_neg);
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

            (
                write_prologue,
                quote_spanned! {op_span=>
                    #get_set
                    let #ident = #input_pos.filter(move |x| !#negset_ident.contains(x));
                },
            )
        };
        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    },
};
