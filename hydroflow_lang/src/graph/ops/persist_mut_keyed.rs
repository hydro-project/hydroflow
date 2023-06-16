use quote::quote_spanned;

use super::{
    DelayType, FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints,
    OperatorWriteOutput, Persistence, WriteContextArgs, RANGE_0, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};
use crate::graph::{OpInstGenerics, OperatorInstance};

/// `persist_mut_keyed()` is similar to `persist_mut()` except that it also enables key-based deletions
/// `persist_mut()` expects an input of type `PersistenceKeyed<T>`, and it is this enumeration that enables the user to communicate deletion.
/// Deletions/persists hapepn in the order they are received in the stream. For example, [Persist(1), Delete(1), Persist(1)] will result ina a single '1' value being stored.
///
/// ```hydroflow
/// source_iter([hydroflow::util::PersistenceKeyed::Persist(0, 1), hydroflow::util::PersistenceKeyed::Persist(1, 1), hydroflow::util::PersistenceKeyed::Delete(1)])
///     -> persist_mut_keyed()
///     -> for_each(|item| println!("{:?}", item));
/// ```
pub const PERSIST_MUT_KEYED: OperatorConstraints = OperatorConstraints {
    name: "persist_mut_keyed",
    categories: &[OperatorCategory::Persistence],
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
        monotonic: FlowPropertyVal::Yes,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| Some(DelayType::Stratum),
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   context,
                   hydroflow,
                   op_span,
                   ident,
                   inputs,
                   is_pull,
                   op_name,
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
        assert!(is_pull);

        if matches!(persistence_args[..], [Persistence::Tick]) {
            diagnostics.push(Diagnostic::spanned(
                op_span,
                Level::Error,
                format!("`{}()` can only have `'static` persistence.", op_name),
            ));
            return Err(());
        };

        let persistdata_ident = wc.make_ident("persistdata");
        let vec_ident = wc.make_ident("persistvec");
        let tick_ident = wc.make_ident("persisttick");
        let write_prologue = quote_spanned! {op_span=>
            let #persistdata_ident = #hydroflow.add_state(::std::cell::RefCell::new((
                0_usize, // tick
                #root::rustc_hash::FxHashMap::<_, #root::util::sparse_vec::SparseVec<_>>::default()
            )));
        };

        let write_iterator = {
            let input = &inputs[0];
            quote_spanned! {op_span=>
                let mut #vec_ident = #context.state_ref(#persistdata_ident).borrow_mut();
                let (ref mut #tick_ident, ref mut #vec_ident) = &mut *#vec_ident;
                let #ident = {
                    #[inline(always)]
                    fn check_iter<K, V>(iter: impl Iterator<Item = #root::util::PersistenceKeyed::<K, V>>) -> impl Iterator<Item = #root::util::PersistenceKeyed::<K, V>> {
                        iter
                    }

                    if *#tick_ident <= #context.current_tick() {
                        *#tick_ident = 1 + #context.current_tick();

                        for item in check_iter(#input) {
                            match item {
                                #root::util::PersistenceKeyed::Persist(k, v) => {
                                    #vec_ident.entry(k).or_default().push(v);
                                },
                                #root::util::PersistenceKeyed::Delete(k) => {
                                    #vec_ident.remove(&k);
                                }
                            }
                        }

                        #[allow(clippy::clone_on_copy)]
                        Some(#vec_ident.iter()
                            .flat_map(|(k, v)| v.iter().map(move |v| (k.clone(), v.clone()))))
                        .into_iter()
                        .flatten()
                    } else {
                        None.into_iter().flatten()
                    }
                };
            }
        };

        let write_iterator_after = quote_spanned! {op_span=>
            #context.schedule_subgraph(#context.current_subgraph(), false);
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
