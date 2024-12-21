use quote::{quote_spanned, ToTokens};
use syn::parse_quote;

use super::{
    DelayType, OpInstGenerics, OperatorCategory, OperatorConstraints, OperatorInstance,
    OperatorWriteOutput, Persistence, PortIndexValue, WriteContextArgs, RANGE_0, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};

// This implementation is largely redundant to ANTI_JOIN and should be DRY'ed
/// > 2 input streams the first of type (K, T), the second of type K,
/// > with output type (K, T)
///
/// For a given tick, computes the anti-join of the items in the input
/// streams, returning items in the `pos` input --that do not have matching keys
/// in the `neg` input. NOTE this uses multiset semantics on the positive side,
/// so duplicated positive inputs will appear in the output either 0 times (if matched in `neg`)
/// or as many times as they appear in the input (if not matched in `neg`)
///
/// ```dfir
/// source_iter(vec![("cat", 2), ("cat", 2), ("elephant", 3), ("elephant", 3)]) -> [pos]diff;
/// source_iter(vec!["dog", "cat", "gorilla"]) -> [neg]diff;
/// diff = anti_join_multiset() -> assert_eq([("elephant", 3), ("elephant", 3)]);
/// ```
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
    // If this is set to true, the state will need to be cleared using `#context.set_state_tick_hook`
    // to prevent reading uncleared data if this subgraph doesn't run.
    // https://github.com/hydro-project/hydroflow/issues/1298
    has_singleton_output: false,
    flo_type: None,
    ports_inn: Some(|| super::PortListSpec::Fixed(parse_quote! { pos, neg })),
    ports_out: None,
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
                        (&mut *#borrow_ident).get_mut_clear(#context.current_tick())
                    },
                ),
                Persistence::Static => (
                    quote_spanned! {op_span=>
                        #root::rustc_hash::FxHashSet::default()
                    },
                    quote_spanned! {op_span=>
                        (&mut *#borrow_ident)
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

        // let vec_ident = wc.make_ident("persistvec");
        let pos_antijoindata_ident = wc.make_ident("antijoindata_pos_ident");
        let pos_borrow_ident = wc.make_ident("antijoindata_pos_borrow_ident");

        let write_prologue_pos = match persistences[0] {
            Persistence::Tick => quote_spanned! {op_span=>},
            Persistence::Static => quote_spanned! {op_span=>
                let #pos_antijoindata_ident = #hydroflow.add_state(std::cell::RefCell::new(
                    ::std::vec::Vec::new()
                ));
            },
            Persistence::Mutable => {
                diagnostics.push(Diagnostic::spanned(
                    op_span,
                    Level::Error,
                    "An implementation of 'mutable does not exist",
                ));
                return Err(());
            }
        };

        let write_prologue = quote_spanned! {op_span=>
            let #neg_antijoindata_ident = #hydroflow.add_state(std::cell::RefCell::new(
                #neg_init
            ));

            #write_prologue_pos
        };

        let input_neg = &inputs[0]; // N before P
        let input_pos = &inputs[1];
        let write_iterator = match persistences[0] {
            Persistence::Tick => quote_spanned! {op_span =>
                let mut #neg_borrow_ident = #context.state_ref(#neg_antijoindata_ident).borrow_mut();

                #[allow(clippy::needless_borrow)]
                #neg_borrow.extend(#input_neg);

                let #ident = #input_pos.filter(|x: &(_,_)| {
                    #[allow(clippy::needless_borrow)]
                    #[allow(clippy::unnecessary_mut_passed)]
                    !#neg_borrow.contains(&x.0)
                });
            },
            Persistence::Static => quote_spanned! {op_span =>
                let mut #neg_borrow_ident = #context.state_ref(#neg_antijoindata_ident).borrow_mut();
                let mut #pos_borrow_ident = #context.state_ref(#pos_antijoindata_ident).borrow_mut();

                #[allow(clippy::needless_borrow)]
                let #ident = {
                    #[allow(clippy::clone_on_copy)]
                    #[allow(suspicious_double_ref_op)]
                    if context.is_first_run_this_tick() {
                        // Start of new tick
                        #neg_borrow.extend(#input_neg);

                        #pos_borrow_ident.extend(#input_pos);
                        #pos_borrow_ident.iter()
                    } else {
                        // Called second or later times on the same tick.
                        let len = #pos_borrow_ident.len();
                        #pos_borrow_ident.extend(#input_pos);
                        #pos_borrow_ident[len..].iter()
                    }
                    .filter(|x: &&(_,_)| {
                        #[allow(clippy::unnecessary_mut_passed)]
                        !#neg_borrow.contains(&x.0)
                    })
                    .map(|(k, v)| (k.clone(), v.clone()))
                };
            },
            Persistence::Mutable => quote_spanned! {op_span =>
                diagnostics.push(Diagnostic::spanned(
                    op_span,
                    Level::Error,
                    "An implementation of 'mutable does not exist",
                ));
                return Err(());
            },
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    },
};
